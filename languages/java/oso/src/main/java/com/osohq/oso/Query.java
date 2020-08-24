package com.osohq.oso;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Field;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.*;
import java.util.stream.Collectors;
import org.json.JSONObject;
import org.json.JSONException;
import org.json.JSONArray;
import org.apache.commons.beanutils.MethodUtils;

public class Query implements Enumeration<HashMap<String, Object>> {
    private HashMap<String, Object> next;
    private Ffi.Query ffiQuery;
    private Host host;
    private Map<Long, Enumeration<Object>> calls;

    /**
     * Construct a new Query object.
     *
     * @param queryPtr Pointer to the FFI query instance.
     */
    public Query(Ffi.Query queryPtr, Host host) throws Exceptions.OsoException {
        this.ffiQuery = queryPtr;
        this.host = host;
        calls = new HashMap<Long, Enumeration<Object>>();
        next = nextResult();
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
            throw new NoSuchElementException("Caused by: e.toString()");
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

    /**
     * Helper for `ExternalCall` query events
     */
    private void handleCall(String attrName, Optional<JSONArray> jArgs, JSONObject polarInstance, long callId)
            throws Exceptions.OsoException {
        Optional<List<Object>> args = Optional.empty();
        if (jArgs.isPresent()) {
            args =  Optional.of(host.polarListToJava(jArgs.get()));
        }
        try {
            registerCall(attrName, args, callId, polarInstance);
        } catch (Exceptions.InvalidCallError e) {
            ffiQuery.applicationError(e.getMessage());
            ffiQuery.callResult(callId, null);
            return;
        }
        String result;
        try {
            result = nextCallResult(callId).toString();
        } catch (NoSuchElementException e) {
            result = null;
        }
        ffiQuery.callResult(callId, result);
    }

    /**
     * Generate the next Query result
     */
    private HashMap<String, Object> nextResult() throws Exceptions.OsoException {
        while (true) {
            String eventStr = ffiQuery.nextEvent().get();
            String kind, className;
            JSONObject data, instance;
            Long callId;

            try {
                JSONObject event = new JSONObject(eventStr);
                kind = event.keys().next();
                data = event.getJSONObject(kind);
            } catch (JSONException e) {
                // TODO: this sucks, we should have a consistent serialization format
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
                    JSONArray initargs;
                    if (constructor.has("InstanceLiteral")) {
                        // Keyword initargs are not supported in Java.
                        className = constructor.getJSONObject("InstanceLiteral").getString("tag");
                        throw new Exceptions.InstantiationError(className);
                    } else if (constructor.has("Call")) {
                        className = constructor.getJSONObject("Call").getString("name");
                        initargs = constructor.getJSONObject("Call").getJSONArray("args");
                    } else {
                        throw new Exceptions.InvalidConstructorError("Bad constructor");
                    }
                    host.makeInstance(className, host.polarListToJava(initargs), id);
                    break;
                case "ExternalCall":
                    instance = data.getJSONObject("instance");
                    callId = data.getLong("call_id");
                    String attrName = data.getString("attribute");

                    Optional<JSONArray> jArgs = Optional.empty();
                    if (!data.get("args").equals(null)) {
                        jArgs = Optional.of(data.getJSONArray("args"));
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
                case "Debug":
                    if (data.has("message")) {
                        String message = data.getString("message");
                        System.out.println(message);
                    }
                    BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
                    System.out.print("debug> ");
                    try {
                        String input = br.readLine();
                        if (input == null)
                            break;
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

    /**
     * Register a Java method call, wrapping the result in an enumeration if it
     * isn't already done.
     *
     * @param attrName      Name of the method/attribute.
     * @param args          Method arguments.
     * @param callId        Call ID under which to register the call.
     * @param polarInstance JSONObject containing either an instance_id or an
     *                      instance of a built-in type.
     */
    public void registerCall(String attrName, Optional<List<Object>> args, long callId, JSONObject polarInstance)
            throws Exceptions.InvalidAttributeError,
                   Exceptions.InvalidCallError,
                   Exceptions.UnregisteredInstanceError,
                   Exceptions.UnexpectedPolarTypeError {
        if (calls.containsKey(callId)) {
            return;
        }
        Object instance;
        if (polarInstance.getJSONObject("value").has("ExternalInstance")) {
            long instanceId = polarInstance.getJSONObject("value").getJSONObject("ExternalInstance")
                    .getLong("instance_id");
            instance = host.getInstance(instanceId);
        } else {
            instance = host.toJava(polarInstance);
        }
        // Select a method to call based on the types of the arguments.
        Object result = null;
        try {
            Class<?> cls = instance instanceof Class ? (Class<?>) instance : instance.getClass();
            if(args.isPresent()) {
                Class<?>[] argTypes = args.get().stream().map(a -> a.getClass())
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
        } catch (IllegalAccessException e) {
            throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
        } catch (InvocationTargetException e) {
            throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
        }
        Enumeration<Object> enumResult;
        if (result instanceof Enumeration) {
            // TODO: test this
            enumResult = (Enumeration<Object>) result;
        } else {
            enumResult = Collections.enumeration(new ArrayList<Object>(Arrays.asList(result)));
        }
        calls.put(callId, enumResult);

    }

    /**
     * Get cached Java method call result.
     */
    private Enumeration<Object> getCall(long callId) throws Exceptions.PolarRuntimeException {
        if (calls.containsKey(callId)) {
            return calls.get(callId);
        } else {
            throw new Exceptions.PolarRuntimeException("Unregistered call ID: " + callId);
        }

    }

    /**
     * Get the next JSONified Polar result of a cached method call (enumeration).
     */
    protected JSONObject nextCallResult(long callId) throws NoSuchElementException, Exceptions.OsoException {
        return host.toPolarTerm(getCall(callId).nextElement());
    }
}
