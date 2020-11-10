package com.osohq.oso;

import java.lang.reflect.Constructor;
import java.util.*;
import java.util.stream.Collectors;
import java.util.stream.DoubleStream;
import java.util.stream.IntStream;
import org.json.JSONArray;
import org.json.JSONObject;

public class Host implements Cloneable {
  private Ffi.Polar ffiPolar;
  private Map<String, Class<?>> classes;
  private Map<Long, Object> instances;

  public Host(Ffi.Polar polarPtr) {
    ffiPolar = polarPtr;
    classes = new HashMap<String, Class<?>>();
    instances = new HashMap<Long, Object>();
  }

  @Override
  public Host clone() {
    Host host = new Host(ffiPolar);
    host.classes.putAll(classes);
    host.instances.putAll(instances);
    return host;
  }

  /** Get a registered Java class. */
  public Class<?> getClass(String name) throws Exceptions.UnregisteredClassError {
    if (classes.containsKey(name)) {
      return classes.get(name);
    } else {
      throw new Exceptions.UnregisteredClassError(name);
    }
  }

  /**
   * Store a Java class in the cache by name.
   *
   * @param name The name used to reference the class from within Polar.
   * @throws Exceptions.DuplicateClassAliasError If the name is already registered.
   */
  public String cacheClass(Class<?> cls, String name) throws Exceptions.DuplicateClassAliasError {
    if (classes.containsKey(name)) {
      throw new Exceptions.DuplicateClassAliasError(
          name, classes.get(name).getName(), cls.getName());
    }
    classes.put(name, cls);
    return name;
  }

  /** Get a cached Java instance. */
  public Object getInstance(long instanceId) throws Exceptions.UnregisteredInstanceError {
    if (hasInstance(instanceId)) {
      return instances.get(instanceId);
    } else {
      throw new Exceptions.UnregisteredInstanceError(instanceId);
    }
  }

  /** Determine if a Java instance has been cached. */
  public boolean hasInstance(long instanceId) {
    return instances.containsKey(instanceId);
  }

  /** Cache an instance of a Java class. */
  public Long cacheInstance(Object instance, Long id) throws Exceptions.OsoException {
    if (id == null) {
      id = ffiPolar.newId();
    }
    instances.put(id, instance);

    return id;
  }

  /** Make an instance of a Java class from a {@code List<Object>} of fields. */
  public Object makeInstance(String className, List<Object> initargs, long id)
      throws Exceptions.OsoException {
    Constructor<?> constructor = null;
    // Try to find a constructor applicable to the supplied arguments.
    Class<?> cls = classes.get(className);
    if (cls == null) {
      throw new Exceptions.UnregisteredClassError(className);
    }
    Class<?>[] argTypes =
        initargs.stream()
            .map(arg -> arg.getClass())
            .collect(Collectors.toUnmodifiableList())
            .toArray(new Class[0]);
    search:
    for (Constructor<?> c : cls.getConstructors()) {
      Class<?>[] paramTypes = c.getParameterTypes();
      if (argTypes.length == paramTypes.length) {
        for (int i = 0; i < paramTypes.length; i++) {
          if (!paramTypes[i].isAssignableFrom(argTypes[i])) {
            continue search;
          }
        }
        constructor = c;
        break search;
      }
    }
    if (constructor == null) throw new Exceptions.MissingConstructorError(className);

    Object instance;
    try {
      instance = constructor.newInstance(initargs.toArray());
    } catch (Exception e) {
      throw new Exceptions.InstantiationError(className, e);
    }
    cacheInstance(instance, id);
    return instance;
  }

  /** Check if a class specializer is more specific than another class specializer. */
  public boolean subspecializer(long instanceId, String leftTag, String rightTag)
      throws Exceptions.UnregisteredClassError, Exceptions.UnregisteredInstanceError {
    Object instance = getInstance(instanceId);
    Class<?> cls, leftClass, rightClass;
    cls = instance.getClass();
    leftClass = getClass(leftTag);
    rightClass = getClass(rightTag);

    if (leftClass.isInstance(instance) || rightClass.isInstance(instance)) {
      while (cls != null) {
        if (cls.equals(leftClass)) {
          return true;
        } else if (cls.equals(rightClass)) {
          return false;
        }
        cls = cls.getSuperclass();
      }
      assert false;
    }
    return false;
  }

  /** Check if a Java instance is an instance of a class. */
  public boolean isa(JSONObject instance, String classTag)
      throws Exceptions.UnregisteredClassError, Exceptions.UnregisteredInstanceError,
          Exceptions.UnexpectedPolarTypeError, Exceptions.OsoException {
    Class<?> cls = getClass(classTag);
    return cls.isInstance(toJava(instance));
  }

  /** Check if two instances unify. */
  public boolean unify(long leftId, long rightId) throws Exceptions.UnregisteredInstanceError {
    Object left = getInstance(leftId);
    Object right = getInstance(rightId);
    if (left == null) {
      return right == null;
    } else {
      return left.equals(right);
    }
  }

  /** Convert Java Objects to Polar (JSON) terms. */
  public JSONObject toPolarTerm(Object value) throws Exceptions.OsoException {
    // Build Polar value
    JSONObject jVal = new JSONObject();
    if (value != null && value.getClass() == Boolean.class) {
      jVal.put("Boolean", value);
    } else if (value != null && value.getClass() == Integer.class) {
      jVal.put("Number", Map.of("Integer", value));
    } else if (value != null
        && (value.getClass() == Float.class || value.getClass() == Double.class)) {
      if ((Double) value == Double.POSITIVE_INFINITY) {
        jVal.put("Number", Map.of("Float", "Infinity"));
      } else if ((Double) value == Double.NEGATIVE_INFINITY) {
        jVal.put("Number", Map.of("Float", "-Infinity"));
      } else if (Double.isNaN((Double) value)) {
        jVal.put("Number", Map.of("Float", "NaN"));
      } else {
        jVal.put("Number", Map.of("Float", value));
      }
    } else if (value != null && value.getClass() == String.class) {
      jVal.put("String", value);
    } else if (value != null && value.getClass().isArray()) {
      jVal.put("List", javaArrayToPolar(value));
    } else if (value != null && value instanceof List) {
      jVal.put("List", javaListToPolar((List<Object>) value));
    } else if (value != null && value instanceof Map) {
      Map<String, JSONObject> jMap = javaMaptoPolar((Map<Object, Object>) value);
      jVal.put("Dictionary", new JSONObject().put("fields", jMap));
    } else if (value != null && value instanceof Predicate) {
      Predicate pred = (Predicate) value;
      if (pred.args == null) pred.args = new ArrayList<Object>();
      jVal.put(
          "Call", new JSONObject(Map.of("name", pred.name, "args", javaListToPolar(pred.args))));
    } else if (value != null && value instanceof Variable) {
      jVal.put("Variable", value);
    } else {
      JSONObject attrs = new JSONObject();
      attrs.put("instance_id", cacheInstance(value, null));
      attrs.put("repr", value == null ? "null" : value.toString());
      jVal.put("ExternalInstance", attrs);
    }

    // Build Polar term
    JSONObject term = new JSONObject();
    term.put("id", 0);
    term.put("offset", 0);
    term.put("value", jVal);
    return term;
  }

  /** Convert a Java List to a JSONified Polar list. */
  private List<JSONObject> javaListToPolar(List<Object> list) throws Exceptions.OsoException {
    ArrayList<JSONObject> polarList = new ArrayList<JSONObject>();
    for (Object el : (List<Object>) list) {
      polarList.add(toPolarTerm(el));
    }
    return polarList;
  }

  /** Convert a Java Array to a JSONified Polar list. */
  private List<JSONObject> javaArrayToPolar(Object array) throws Exceptions.OsoException {
    assert (array.getClass().isArray());

    List<Object> l;
    if (array instanceof int[]
        || array instanceof boolean[]
        || array instanceof char[]
        || array instanceof byte[]) {
      l = IntStream.of((int[]) array).boxed().collect(Collectors.toList());

    } else if (array instanceof float[] || array instanceof double[]) {
      l = DoubleStream.of((double[]) array).boxed().collect(Collectors.toList());

    } else if (array instanceof Object[]) {
      l = Arrays.asList((Object[]) array);

    } else {
      throw new Exceptions.OsoException(
          "Oso does not support arrays of type " + array.getClass().getComponentType().getName());
    }
    return javaListToPolar(l);
  }

  /** Convert a Java Map to a JSONified Polar dictionary. */
  private Map<String, JSONObject> javaMaptoPolar(Map<Object, Object> map)
      throws Exceptions.OsoException {
    HashMap<String, JSONObject> polarDict = new HashMap<String, JSONObject>();
    for (Object key : map.keySet()) {
      JSONObject val = toPolarTerm(map.get(key));
      polarDict.put(key.toString(), val);
    }
    return polarDict;
  }

  /** Turn a Polar term passed across the FFI boundary into a Java Object. */
  public Object toJava(JSONObject term)
      throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    JSONObject value = term.getJSONObject("value");
    String tag = value.keys().next();
    switch (tag) {
      case "String":
        return value.getString(tag);
      case "Boolean":
        return value.getBoolean(tag);
      case "Number":
        JSONObject num = value.getJSONObject(tag);
        switch (num.keys().next()) {
          case "Integer":
            return num.getInt("Integer");
          case "Float":
            Object f = num.get("Float");
            if (f instanceof String) {
              switch ((String) f) {
                case "Infinity":
                  return Double.POSITIVE_INFINITY;
                case "-Infinity":
                  return Double.NEGATIVE_INFINITY;
                case "NaN":
                  return Double.NaN;
                default:
                  throw new Exceptions.OsoException(
                      "Expected a floating point number, got \"" + f + "\"");
              }
            }
            return (Double) f;
        }
      case "List":
        return polarListToJava(value.getJSONArray(tag));
      case "Dictionary":
        return polarDictToJava(value.getJSONObject(tag).getJSONObject("fields"));
      case "ExternalInstance":
        return getInstance(value.getJSONObject(tag).getLong("instance_id"));
      case "Call":
        List<Object> args = polarListToJava(value.getJSONObject(tag).getJSONArray("args"));
        return new Predicate(value.getJSONObject(tag).getString("name"), args);
      case "Variable":
        return new Variable(value.getString(tag));
      default:
        throw new Exceptions.UnexpectedPolarTypeError(tag);
    }
  }

  /** Convert a JSONified Polar dictionary to a Java Map */
  public HashMap<String, Object> polarDictToJava(JSONObject dict)
      throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    HashMap<String, Object> resMap = new HashMap<String, Object>();
    for (String key : dict.keySet()) {
      resMap.put(key, toJava(dict.getJSONObject(key)));
    }
    return resMap;
  }

  /** Convert a JSONified Polar List to a Java List */
  public List<Object> polarListToJava(JSONArray list)
      throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    ArrayList<Object> resArray = new ArrayList<Object>();
    for (int i = 0; i < list.length(); i++) {
      resArray.add(toJava(list.getJSONObject(i)));
    }
    return resArray;
  }
}
