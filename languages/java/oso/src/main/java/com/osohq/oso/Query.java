package com.osohq.oso;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.Collection;
import java.util.Collections;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.Optional;
import java.util.stream.Collectors;
import org.apache.commons.beanutils.MethodUtils;
import org.apache.commons.collections4.IteratorUtils;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

public class Query implements Enumeration<HashMap<String, Object>> {
  /**
   * The next result to return from the query.
   *
   * <p>Since query implements `Enumeration` we must detect whether there is another result before
   * `hasMoreElements()` is called.
   *
   * <p>To do this, we call `nextResult` before the result is needed by the user, storing it in
   * `next`.
   *
   * <p>If `next` is `null`, there are no more results.
   */
  private HashMap<String, Object> next;

  private Ffi.Query ffiQuery;
  private Host host;
  private Map<Long, Enumeration<Object>> calls;

  /**
   * Construct a new Query object.
   *
   * @param queryPtr Pointer to the FFI query instance.
   */
  public Query(Ffi.Query queryPtr, Host host, Map<String, Object> bindings)
      throws Exceptions.OsoException {
    this.ffiQuery = queryPtr;
    this.host = host;
    calls = new HashMap<Long, Enumeration<Object>>();

    for (Map.Entry<String, Object> binding : bindings.entrySet()) {
      bind(binding.getKey(), binding.getValue());
    }

    // Get the first result of the query. Must run after initialization.
    next = nextResult();
  }

  private void bind(String name, Object value) {
    this.ffiQuery.bind(name, this.host.toPolarTerm(value).toString());
  }

  @Override
  public boolean hasMoreElements() {
    return next != null;
  }

  @Override
  public HashMap<String, Object> nextElement() {
    HashMap<String, Object> ret = next;
    try {
      next = nextResult();
    } catch (Exception e) {
      throw new NoSuchElementException("Caused by: " + e.toString());
    }
    return ret;
  }

  /**
   * Get all query results
   *
   * @return List of all query results (binding sets)
   */
  public List<HashMap<String, Object>> results() {
    List<HashMap<String, Object>> results = Collections.list(this);
    return results;
  }

  /** Helper for `ExternalCall` query events */
  private void handleCall(
      String attrName, Optional<JSONArray> jArgs, JSONObject polarInstance, long callId)
      throws Exceptions.OsoException {
    Optional<List<Object>> args = Optional.empty();
    if (jArgs.isPresent()) {
      args = Optional.of(host.polarListToJava(jArgs.get()));
    }
    try {
      Object instance = host.toJava(polarInstance);
      // Select a method to call based on the types of the arguments.
      Object result = null;
      try {
        Class<?> cls = instance instanceof Class ? (Class<?>) instance : instance.getClass();
        if (args.isPresent()) {
          Class<?>[] argTypes =
              args.get().stream()
                  .map(a -> a.getClass())
                  .collect(Collectors.toUnmodifiableList())
                  .toArray(new Class[0]);
          Method method = MethodUtils.getMatchingAccessibleMethod(cls, attrName, argTypes);
          if (method == null) {
            throw new Exceptions.InvalidCallError(cls.getName(), attrName, argTypes);
          }
          result = method.invoke(instance, args.get().toArray());
        } else {
          // Look for a field with the given name.
          try {
            Field field = cls.getField(attrName);
            result = field.get(instance);
          } catch (NoSuchFieldException f) {
            throw new Exceptions.InvalidAttributeError(cls.getName(), attrName);
          }
        }
        String term = host.toPolarTerm(result).toString();
        ffiQuery.callResult(callId, term);

      } catch (IllegalAccessException e) {
        throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
      } catch (InvocationTargetException e) {
        throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
      }
    } catch (Exceptions.InvalidCallError e) {
      ffiQuery.applicationError(e.getMessage());
      ffiQuery.callResult(callId, "null");
      return;
    } catch (Exceptions.InvalidAttributeError e) {
      ffiQuery.applicationError(e.getMessage());
      ffiQuery.callResult(callId, "null");
      return;
    }
  }

  /** Helper for `NextExternal` query events */
  private void handleNextExternal(long callId, JSONObject iterable) throws Exceptions.OsoException {
    if (!calls.containsKey(callId)) {
      Object result = host.toJava(iterable);
      Enumeration<Object> enumResult;
      if (result instanceof Enumeration<?>) {
        enumResult = (Enumeration<Object>) result;
      } else if (result instanceof Collection<?>) {
        enumResult = java.util.Collections.enumeration((Collection<Object>) result);
      } else if (result instanceof Iterable<?>) {
        enumResult = IteratorUtils.asEnumeration(((Iterable<?>) result).iterator());
      } else {
        throw new Exceptions.InvalidIteratorError(
            String.format("value %s of type %s is not iterable", result, result.getClass()));
      }
      calls.put(callId, enumResult);
    }
    String result;

    try {
      result = nextCallResult(callId).toString();
    } catch (NoSuchElementException e) {
      result = "null";
    }
    ffiQuery.callResult(callId, result);
  }

  /** Generate the next Query result */
  private HashMap<String, Object> nextResult() throws Exceptions.OsoException {
    while (true) {
      String eventStr = ffiQuery.nextEvent();
      String kind, className;
      JSONObject data, instance;
      Long callId;

      try {
        JSONObject event = new JSONObject(eventStr);
        kind = event.keys().next();
        data = event.getJSONObject(kind);
      } catch (JSONException e) {
        // TODO: we should have a consistent serialization format
        kind = eventStr.replace("\"", "");
        data = null;
      }

      switch (kind) {
        case "Done":
          return null;
        case "Result":
          return host.polarDictToJava(data.getJSONObject("bindings"));
        case "MakeExternal":
          Long id = data.getLong("instance_id");
          if (host.hasInstance(id)) {
            throw new Exceptions.DuplicateInstanceRegistrationError(id);
          }

          JSONObject constructor = data.getJSONObject("constructor").getJSONObject("value");
          if (constructor.has("Call")) {
            className = constructor.getJSONObject("Call").getString("name");
            JSONArray initargs = constructor.getJSONObject("Call").getJSONArray("args");

            // kwargs should always be null in Java
            if (constructor.getJSONObject("Call").get("kwargs") != JSONObject.NULL) {
              throw new Exceptions.InstantiationError(className);
            }
            host.makeInstance(className, host.polarListToJava(initargs), id);
            break;
          } else {
            throw new Exceptions.InvalidConstructorError("Bad constructor");
          }
        case "ExternalCall":
          instance = data.getJSONObject("instance");
          callId = data.getLong("call_id");
          String attrName = data.getString("attribute");

          Optional<JSONArray> jArgs = Optional.empty();
          if (!data.get("args").equals(null)) {
            jArgs = Optional.of(data.getJSONArray("args"));
          }
          if (!data.get("kwargs").equals(null)) {
            throw new Exceptions.InvalidCallError("Java does not support keyword arguments");
          }
          handleCall(attrName, jArgs, instance, callId);
          break;
        case "ExternalIsa":
          instance = data.getJSONObject("instance");
          callId = data.getLong("call_id");
          className = data.getString("class_tag");
          int answer = host.isa(instance, className) ? 1 : 0;
          ffiQuery.questionResult(callId, answer);
          break;
        case "ExternalIsSubSpecializer":
          long instanceId = data.getLong("instance_id");
          callId = data.getLong("call_id");
          String leftTag = data.getString("left_class_tag");
          String rightTag = data.getString("right_class_tag");
          answer = host.subspecializer(instanceId, leftTag, rightTag) ? 1 : 0;
          ffiQuery.questionResult(callId, answer);
          break;
        case "ExternalIsSubclass":
          callId = data.getLong("call_id");
          answer =
              host.isSubclass(data.getString("left_class_tag"), data.getString("right_class_tag"))
                  ? 1
                  : 0;
          ffiQuery.questionResult(callId, answer);
          break;
        case "ExternalOp":
          callId = data.getLong("call_id");
          JSONArray args = data.getJSONArray("args");
          answer = host.operator(data.getString("operator"), host.polarListToJava(args)) ? 1 : 0;
          ffiQuery.questionResult(callId, answer);
          break;
        case "NextExternal":
          callId = data.getLong("call_id");
          JSONObject iterable = data.getJSONObject("iterable");
          handleNextExternal(callId, iterable);
          break;
        case "Debug":
          if (data.has("message")) {
            String message = data.getString("message");
            System.out.println(message);
          }
          BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
          System.out.print("debug> ");
          try {
            String input = br.readLine();
            if (input == null) break;
            String command = host.toPolarTerm(input).toString();
            ffiQuery.debugCommand(command);
          } catch (IOException e) {
            throw new Exceptions.PolarRuntimeException("Caused by: " + e.getMessage());
          }
          break;
        default:
          throw new Exceptions.PolarRuntimeException("Unhandled event type: " + kind);
      }
    }
  }

  /** Get cached Java method call result. */
  private Enumeration<Object> getCall(long callId) throws Exceptions.PolarRuntimeException {
    if (calls.containsKey(callId)) {
      return calls.get(callId);
    } else {
      throw new Exceptions.PolarRuntimeException("Unregistered call ID: " + callId);
    }
  }

  /** Get the next JSONified Polar result of a cached method call (enumeration). */
  protected JSONObject nextCallResult(long callId)
      throws NoSuchElementException, Exceptions.OsoException {
    return host.toPolarTerm(getCall(callId).nextElement());
  }
}
