package com.osohq.oso;

import java.lang.reflect.Constructor;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.DoubleStream;
import java.util.stream.IntStream;
import org.json.JSONArray;
import org.json.JSONObject;

public class Host implements Cloneable {
  private Ffi.Polar ffiPolar;
  private Map<String, Class<?>> classes;
  private Map<Class<?>, Long> classIds;
  private Map<Long, Object> instances;

  // Set to true to accept an expression from the core in toJava.
  protected boolean acceptExpression;

  public Host(Ffi.Polar polarPtr) {
    acceptExpression = false;
    ffiPolar = polarPtr;
    classes = new HashMap<String, Class<?>>();
    classIds = new HashMap<Class<?>, Long>();
    instances = new HashMap<Long, Object>();
  }

  @Override
  public Host clone() {
    Host host = new Host(ffiPolar);
    host.classes.putAll(classes);
    host.classIds.putAll(classIds);
    host.instances.putAll(instances);
    host.acceptExpression = acceptExpression;
    return host;
  }

  protected void setAcceptExpression(boolean acceptExpression) {
    this.acceptExpression = acceptExpression;
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
    classIds.put(cls, cacheInstance(cls, null));
    return name;
  }

  /**
   * Register a list of base classes (MRO list) for all registered classes. The list is in method
   * resolution order (MRO), meaning the superclasses are ordered from most to least specific.
   */
  public void registerMros() {

    for (Map.Entry<String, Class<?>> cls : classes.entrySet()) {
      Class<?> scls = cls.getValue().getSuperclass();
      List<Long> mro = new ArrayList<Long>();
      while (scls != null) {
        Long id = classIds.get(scls);
        if (id != null) mro.add(id);
        scls = scls.getSuperclass();
      }
      ffiPolar.registerMro(cls.getKey(), mro.toString());
    }
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
      throws Exceptions.UnregisteredClassError,
          Exceptions.UnregisteredInstanceError,
          Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    Class<?> cls = getClass(classTag);
    return cls.isInstance(toJava(instance));
  }

  /** Return true if left is a subclass (or the same class) as right. */
  public boolean isSubclass(String leftTag, String rightTag) {
    Class<?> leftClass, rightClass;
    leftClass = getClass(leftTag);
    rightClass = getClass(rightTag);

    return rightClass.isAssignableFrom(leftClass);
  }

  public boolean operator(String op, List<Object> args) throws Exceptions.OsoException {
    Object left = args.get(0), right = args.get(1);
    if (op.equals("Eq")) {
      if (left == null) return left == right;
      else return left.equals(right);
    }
    throw new Exceptions.UnimplementedOperation(op);
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
      Map<Object, Object> valueMap = (Map<Object, Object>) value;
      HashMap<String, Object> stringMap = new HashMap<String, Object>();

      // Polar only supports dictionaries with string keys. Convert a map to a map of
      // string keys.
      for (Object objectKey : valueMap.keySet()) {
        if (!(objectKey instanceof String)) {
          throw new Exceptions.UnexpectedPolarTypeError(
              "Cannot convert map with non-string keys to Polar");
        }
        String key = (String) objectKey;
        stringMap.put(key, valueMap.get(objectKey));
      }

      Map<String, JSONObject> jMap = javaMaptoPolar(stringMap);
      jVal.put("Dictionary", new JSONObject().put("fields", jMap));
    } else if (value != null && value instanceof Predicate) {
      Predicate pred = (Predicate) value;
      if (pred.args == null) pred.args = new ArrayList<Object>();
      jVal.put(
          "Call", new JSONObject(Map.of("name", pred.name, "args", javaListToPolar(pred.args))));
    } else if (value != null && value instanceof Variable) {
      jVal.put("Variable", value);
    } else if (value != null && value instanceof Expression) {
      Expression expression = (Expression) value;
      JSONObject expressionJSON = new JSONObject();
      expressionJSON.put("operator", expression.getOperator().toString());
      expressionJSON.put("args", javaListToPolar(expression.getArgs()));
      jVal.put("Expression", expressionJSON);
    } else if (value != null && value instanceof Pattern) {
      Pattern pattern = (Pattern) value;
      if (pattern.getTag() == null) {
        jVal.put("Pattern", toPolarTerm(pattern.getFields()));
      } else {
        JSONObject fieldsJSON = new JSONObject();
        fieldsJSON.put("fields", javaMaptoPolar(pattern.getFields()));

        JSONObject instanceJSON = new JSONObject();
        instanceJSON.put("tag", pattern.getTag());
        instanceJSON.put("fields", fieldsJSON);

        JSONObject patternJSON = new JSONObject();
        patternJSON.put("Instance", instanceJSON);

        jVal.put("Pattern", patternJSON);
      }
    } else {
      JSONObject attrs = new JSONObject();
      Long instanceId = null;

      // if the object is a Class, then it will already have an instance ID
      if (value instanceof Class) {
        instanceId = classIds.get(value);
      }

      attrs.put("instance_id", cacheInstance(value, instanceId));
      attrs.put("repr", value == null ? "null" : value.toString());

      // pass a class_repr string *for registered types only*
      if (value != null) {
        Class classFromValue = value.getClass();
        String stringifiedClassFromValue = classFromValue.toString();
        stringifiedClassFromValue =
            classIds.containsKey(classFromValue) ? stringifiedClassFromValue : "null";
        attrs.put("class_repr", stringifiedClassFromValue);
      } else {
        attrs.put("class_repr", "null");
      }

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
  private Map<String, JSONObject> javaMaptoPolar(Map<String, Object> map)
      throws Exceptions.OsoException {
    HashMap<String, JSONObject> polarDict = new HashMap<String, JSONObject>();
    for (String key : map.keySet()) {
      JSONObject val = toPolarTerm(map.get(key));
      polarDict.put(key, val);
    }
    return polarDict;
  }

  /** Turn a Polar term passed across the FFI boundary into a Java Object. */
  public Object toJava(JSONObject term)
      throws Exceptions.UnregisteredInstanceError,
          Exceptions.UnexpectedPolarTypeError,
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
      case "Expression":
        if (!this.acceptExpression) {
          throw new Exceptions.UnexpectedPolarTypeError(Exceptions.UNEXPECTED_EXPRESSION_MESSAGE);
        }
        return new Expression(
            value.getJSONObject(tag).getEnum(Operator.class, "operator"),
            polarListToJava(value.getJSONObject(tag).getJSONArray("args")));
      case "Pattern":
        JSONObject pattern = value.getJSONObject("Pattern");
        String patternTag = pattern.keys().next();
        JSONObject patternValue = pattern.getJSONObject(patternTag);
        switch (patternTag) {
          case "Instance":
            return new Pattern(
                patternValue.getString("tag"),
                polarDictToJava(patternValue.getJSONObject("fields").getJSONObject("fields")));
          case "Dictionary":
            return new Pattern(null, polarDictToJava(patternValue));
          default:
            throw new Exceptions.UnexpectedPolarTypeError("Pattern: " + patternTag);
        }
      default:
        throw new Exceptions.UnexpectedPolarTypeError(tag);
    }
  }

  /** Convert a JSONified Polar dictionary to a Java Map */
  public HashMap<String, Object> polarDictToJava(JSONObject dict)
      throws Exceptions.UnregisteredInstanceError,
          Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    HashMap<String, Object> resMap = new HashMap<String, Object>();
    for (String key : dict.keySet()) {
      resMap.put(key, toJava(dict.getJSONObject(key)));
    }
    return resMap;
  }

  /** Convert a JSONified Polar List to a Java List */
  public List<Object> polarListToJava(JSONArray list)
      throws Exceptions.UnregisteredInstanceError,
          Exceptions.UnexpectedPolarTypeError,
          Exceptions.OsoException {
    ArrayList<Object> resArray = new ArrayList<Object>();
    for (int i = 0; i < list.length(); i++) {
      resArray.add(toJava(list.getJSONObject(i)));
    }
    return resArray;
  }
}
