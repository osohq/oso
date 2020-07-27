package com.osohq.oso;

import java.util.*;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.DoubleStream;
import java.util.stream.IntStream;
import org.json.JSONObject;
import org.json.JSONArray;

public class Host implements Cloneable {
    private Ffi.Polar ffiPolar;
    private Map<String, Class<Object>> classes;
    private Map<String, Function<Map, Object>> constructors;
    private Map<Long, Object> instances;

    public Host(Ffi.Polar polarPtr) {
        ffiPolar = polarPtr;
        classes = new HashMap<String, Class<Object>>();
        constructors = new HashMap<String, Function<Map, Object>>();
        instances = new HashMap<Long, Object>();
    }

    @Override
    public Host clone() {
        Host host = new Host(ffiPolar);
        host.classes.putAll(classes);
        host.constructors.putAll(constructors);
        host.instances.putAll(instances);
        return host;
    }

    /**
     * Get a registered Java class.
     *
     * @param name
     * @throws Exceptions.UnregisteredClassError
     */
    public Class getClass(String name) throws Exceptions.UnregisteredClassError {
        if (classes.containsKey(name)) {
            return classes.get(name);
        } else {
            throw new Exceptions.UnregisteredClassError(name);
        }
    }

    /**
     * Store a Java class in the cache by name.
     *
     * @param name
     * @throws Exceptions.DuplicateClassAliasError If the class is already
     *                                             registered.
     */
    public String cacheClass(Class cls, Function<Map, Object> constructor, String name)
            throws Exceptions.DuplicateClassAliasError {
        if (classes.containsKey(name)) {
            throw new Exceptions.DuplicateClassAliasError(name, classes.get(name).getName(), cls.getName());
        }
        classes.put(name, cls);
        constructors.put(name, constructor);
        return name;
    }

    /**
     * Get a cached Java instance.
     *
     * @param instanceId
     * @throws Exceptions.UnregisteredInstanceError
     */
    public Object getInstance(long instanceId) throws Exceptions.UnregisteredInstanceError {
        if (hasInstance(instanceId)) {
            return instances.get(instanceId);
        } else {
            throw new Exceptions.UnregisteredInstanceError(instanceId);
        }
    }

    /**
     * Determine if a Java instance has been cached.
     *
     * @param instanceId
     */
    public boolean hasInstance(long instanceId) {
        return instances.containsKey(instanceId);
    }

    /**
     * Cache an instance of a Java class.
     *
     * @param instance
     * @param id
     * @throws Exceptions.OsoException
     */
    public Long cacheInstance(Object instance, Long id) throws Exceptions.OsoException {
        if (id == null) {
            id = ffiPolar.newId();
        }
        instances.put(id, instance);

        return id;
    }

    /**
     * Make an instance of a Java class from a {@code Map<String, Object>} of
     * fields.
     *
     * @param clsName
     * @param fields
     * @param id
     */
    public Object makeInstance(String clsName, Map fields, long id) throws Exceptions.OsoException {
        Function<Map, Object> constructor = constructors.get(clsName);
        Object instance;
        if (constructor != null) {
            instance = constructor.apply(fields);
        } else {
            // TODO: default constructor
            throw new Exceptions.MissingConstructorError(clsName);
        }
        cacheInstance(instance, id);
        return instance;
    }

    /**
     * Check if a class specializer is more specific than another class specializer.
     *
     * @param instanceId
     * @param leftTag
     * @param rightTag
     * @return
     * @throws Exceptions.UnregisteredClassError
     */
    public boolean subspecializer(long instanceId, String leftTag, String rightTag)
            throws Exceptions.UnregisteredClassError {
        Object instance = instances.get(instanceId);
        Class cls, leftClass, rightClass;
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

    /**
     * Check if a Java instance is an instance of a class.
     *
     * @param instanceId
     * @param classTag
     * @return
     * @throws Exceptions.UnregisteredClassError
     * @throws Exceptions.UnregisteredInstanceError
     */
    public boolean isa(long instanceId, String classTag)
            throws Exceptions.UnregisteredClassError, Exceptions.UnregisteredInstanceError {
        Class cls = getClass(classTag);
        Object instance = getInstance(instanceId);
        return cls.isInstance(instance);
    }

    /**
     * Convert Java Objects to Polar (JSON) terms.
     *
     * @param value Java Object to be converted to Polar.
     * @return JSONObject Polar term of form: {@code {"id": _, "offset": _, "value":
     *         _}}.
     */
    public JSONObject toPolarTerm(Object value) throws Exceptions.OsoException {
        // Build Polar value
        JSONObject jVal = new JSONObject();
        if (value != null && value.getClass() == Boolean.class) {
            jVal.put("Boolean", value);
        } else if (value != null && value.getClass() == Integer.class) {
            jVal.put("Number", Map.of("Integer", value));
        } else if (value != null && (value.getClass() == Float.class || value.getClass() == Double.class)) {
            jVal.put("Number", Map.of("Float", value));
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
            if (pred.args == null)
                pred.args = new ArrayList<Object>();
            jVal.put("Call", new JSONObject(Map.of("name", pred.name, "args", javaListToPolar(pred.args))));
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

    /**
     * Convert a Java List to a JSONified Polar list.
     *
     * @param list List<Object>
     * @return List<JSONObject>
     * @throws Exceptions.OsoException
     */
    private List<JSONObject> javaListToPolar(List<Object> list) throws Exceptions.OsoException {
        ArrayList<JSONObject> polarList = new ArrayList<JSONObject>();
        for (Object el : (List<Object>) list) {
            polarList.add(toPolarTerm(el));
        }
        return polarList;
    }

    /**
     * Convert a Java Array to a JSONified Polar list.
     *
     * @param list List<Object>
     * @return List<JSONObject>
     * @throws Exceptions.OsoException
     */
    private List<JSONObject> javaArrayToPolar(Object array) throws Exceptions.OsoException {
        assert (array.getClass().isArray());

        List<Object> l;
        if (array instanceof int[] || array instanceof boolean[] || array instanceof char[]
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

    /**
     * Convert a Java Map to a JSONified Polar dictionary.
     *
     * @param map Java Map<Object, Object>
     * @return Map<String, JSONObject>
     * @throws Exceptions.OsoException
     */
    private Map<String, JSONObject> javaMaptoPolar(Map<Object, Object> map) throws Exceptions.OsoException {
        HashMap<String, JSONObject> polarDict = new HashMap<String, JSONObject>();
        for (Object key : map.keySet()) {
            JSONObject val = toPolarTerm(map.get(key));
            polarDict.put(key.toString(), val);
        }
        return polarDict;
    }

    /**
     * Turn a Polar term passed across the FFI boundary into a Java Object.
     *
     * @param term JSONified Polar term of the form: {@code {"id": _, "offset": _,
     *             "value": _}}
     * @throws Exceptions.UnregisteredInstanceError
     * @throws Exceptions.UnexpectedPolarTypeError
     */
    public Object toJava(JSONObject term)
            throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError {
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
                        return num.getFloat("Float");
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

    /**
     * Convert a JSONified Polar dictionary to a Java Map
     *
     * @param dict JSONObject
     * @throws Exceptions.UnregisteredInstanceError
     * @throws Exceptions.UnexpectedPolarTypeError
     */
    public HashMap<String, Object> polarDictToJava(JSONObject dict)
            throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError {
        HashMap<String, Object> resMap = new HashMap<String, Object>();
        for (String key : dict.keySet()) {
            resMap.put(key, toJava(dict.getJSONObject(key)));
        }
        return resMap;
    }

    /**
     * Convert a JSONified Polar List to a Java List
     *
     * @param list JSONArray
     */
    public List<Object> polarListToJava(JSONArray list)
            throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError {
        ArrayList<Object> resArray = new ArrayList<Object>();
        for (int i = 0; i < list.length(); i++) {
            resArray.add(toJava(list.getJSONObject(i)));
        }
        return resArray;
    }
}
