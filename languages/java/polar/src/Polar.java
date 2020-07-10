import org.json.*;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.*;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Field;

public class Polar {
    private Ffi.Polar ffi;
    private Map<String, Class<Object>> classes;
    private Map<String, Function<Map, Object>> constructors;
    private Map<Long, Object> instances;
    private Map<String, String> loadQueue; // Map from filename -> file contents
    private Map<Long, Enumeration<Object>> calls;

    public Polar() throws Exceptions.OsoException {
        ffi = new Ffi().polarNew();
        classes = new HashMap<String, Class<Object>>();
        constructors = new HashMap<String, Function<Map, Object>>();
        instances = new HashMap<Long, Object>();
        loadQueue = new HashMap<String, String>();
        calls = new HashMap<Long, Enumeration<Object>>();
    }

    /*********************/
    /* PROTECTED METHODS */
    /*********************/

    /**
     * Enqueue a polar policy file to be loaded. File contents are loaded into a
     * String and saved here, so changes to the file made after calls to loadFile
     * will not be recognized. If the filename already exists in the load queue,
     * replace it.
     *
     * @param filename
     * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
     * @throws IOException                        If unable to open or read the
     *                                            file.
     */
    protected void loadFile(String filename) throws IOException, Exceptions.PolarFileExtensionError {
        Optional<String> ext = Optional.ofNullable(filename).filter(f -> f.contains("."))
                .map(f -> f.substring(filename.lastIndexOf(".") + 1));

        // check file extension
        if (!ext.isPresent() || !ext.get().equals("polar")) {
            throw new Exceptions.PolarFileExtensionError();
        }

        // add file to queue
        loadQueue.put(filename, new String(Files.readAllBytes(Paths.get(filename))));
    }

    /**
     * Clear the KB, but maintain all registered classes and calls
     *
     * @throws Exceptions.OsoException
     */
    protected void clear() throws Exceptions.OsoException {
        // clear all Queued files
        loadQueue.clear();

        // Replace Polar instance
        ffi = new Ffi().polarNew();
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     * @throws Exceptions.OsoException
     */
    protected void loadStr(String str, String filename) throws Exceptions.OsoException {
        ffi.loadStr(str, filename);
        checkInlineQueries();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     * @throws Exceptions.OsoException
     */
    protected void loadStr(String str) throws Exceptions.OsoException {
        ffi.loadStr(str, null);
        checkInlineQueries();

    }

    /**
     * Query for a Polar string.
     *
     * @param queryStr Query string
     * @return Query object (Enumeration of resulting variable bindings).
     */
    protected Query queryStr(String queryStr) throws Exceptions.OsoException {
        clearQueryState();
        loadQueuedFiles();
        return new Query(ffi.newQueryFromStr(queryStr), this);
    }

    /**
     * Query for a Predicate.
     *
     * @param name Predicate name, e.g. "f" for predicate "f(x)".
     * @param args List of predicate arguments.
     * @return Query object (Enumeration of resulting variable bindings).
     * @throws Exceptions.OsoException
     */
    protected Query queryPred(String name, List<Object> args) throws Exceptions.OsoException {
        clearQueryState();
        loadQueuedFiles();
        String pred = toPolarTerm(new Predicate(name, args)).toString();
        return new Query(ffi.newQueryFromTerm(pred), this);
    }

    /**
     * Start the Polar REPL.
     *
     * @throws Exceptions.OsoException
     */
    protected void repl() throws Exceptions.OsoException {
        // clear_query_state
        loadQueuedFiles();
        while (true) {
            Query query = new Query(ffi.newQueryFromRepl(), this);
            if (!query.hasMoreElements()) {
                System.out.println("False");
            } else {
                do {
                    System.out.println(query.nextElement());
                } while (query.hasMoreElements());
            }

        }
    }

    /**
     * Register a Java class with oso.
     *
     * @param cls       Class object to be registered.
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @throws Exceptions.DuplicateClassAliasError if class has already been
     *                                             registered.
     */
    protected void registerClass(Class cls, Function<Map, Object> fromPolar)
            throws Exceptions.DuplicateClassAliasError {
        String name = cls.getName();
        if (classes.containsKey(name)) {
            throw new Exceptions.DuplicateClassAliasError(name, classes.get(name).getName(), name);
        }
        classes.put(cls.getName(), cls);
        constructors.put(cls.getName(), fromPolar);
    }

    /**
     * Register a Java class with oso using an alias.
     *
     * @param cls       Class object to be registered.
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @param alias     name to register the class under, which is how the class is
     *                  accessed from Polar.
     * @throws Exceptions.DuplicateClassAliasError if a class has already been
     *                                             registered with the given alias.
     */
    protected void registerClass(Class cls, Function<Map, Object> fromPolar, String alias)
            throws Exceptions.DuplicateClassAliasError {
        if (classes.containsKey(alias)) {
            throw new Exceptions.DuplicateClassAliasError(alias, classes.get(alias).getName(), cls.getName());
        }
        classes.put(alias, cls);
        constructors.put(alias, fromPolar);
    }

    /**
     * Convert Java Objects to Polar (JSON) terms.
     *
     * @param value Java Object to be converted to Polar.
     * @return JSONObject Polar term of form: {@code {"id": _, "offset": _, "value":
     *         _}}.
     */
    protected JSONObject toPolarTerm(Object value) throws Exceptions.OsoException {
        // Build Polar value
        JSONObject jVal = new JSONObject();
        if (value.getClass() == Boolean.class) {
            jVal.put("Boolean", value);

        } else if (value.getClass() == Integer.class) {
            jVal.put("Number", Map.of("Integer", value));

        } else if (value.getClass() == Float.class) {
            jVal.put("Number", Map.of("Float", value));

        } else if (value.getClass() == String.class) {
            jVal.put("String", value);

        } else if (value instanceof List) {
            jVal.put("List", javaListToPolar((List<Object>) value));

        } else if (value instanceof Map) {
            Map<String, JSONObject> jMap = javaMaptoPolar((Map<Object, Object>) value);
            jVal.put("Dictionary", new JSONObject().put("fields", jMap));

        } else if (value instanceof Predicate) {
            Predicate pred = (Predicate) value;
            jVal.put("Call", new JSONObject(Map.of("name", pred.name, "args", javaListToPolar(pred.args))));
        } else if (value instanceof Variable) {
            jVal.put("Symbol", value);
        } else {
            jVal.put("ExternalInstance", new JSONObject().put("instance_id", cacheInstance(value, null)));
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
     * @return Object
     * @throws Exceptions.UnregisteredInstanceError
     * @throws Exceptions.UnexpectedPolarTypeError
     */
    protected Object toJava(JSONObject term)
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
                return getCachedInstance(value.getJSONObject(tag).getLong("instance_id"));
            case "Call":
                List<Object> args = polarListToJava(value.getJSONObject(tag).getJSONArray("args"));
                return new Predicate(value.getJSONObject(tag).getString("name"), args);
            default:
                throw new Exceptions.UnexpectedPolarTypeError(tag);
        }
    }

    /**
     * Convert a JSONified Polar dictionary to a Java Map
     *
     * @param dict JSONObject
     * @return HashMap<String, Object>
     * @throws Exceptions.UnregisteredInstanceError
     * @throws Exceptions.UnexpectedPolarTypeError
     */
    protected HashMap<String, Object> polarDictToJava(JSONObject dict)
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
     * @return List<Object>
     * @throws Exceptions.UnregisteredInstanceError
     * @throws Exceptions.UnexpectedPolarTypeError
     */
    protected List<Object> polarListToJava(JSONArray list)
            throws Exceptions.UnregisteredInstanceError, Exceptions.UnexpectedPolarTypeError {
        ArrayList<Object> resArray = new ArrayList<Object>();
        for (int i = 0; i < list.length(); i++) {
            resArray.add(toJava(list.getJSONObject(i)));
        }
        return resArray;

    }

    /**
     * Make an instance of a Java class from a {@code Map<String, Object>} of
     * fields.
     *
     * @param clsName
     * @param fields
     * @param id
     * @return Object
     */
    protected Object makeInstance(String clsName, Map fields, long id) throws Exceptions.OsoException {
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
     * Cache an instance of a Java class.
     *
     * @param instance
     * @param id
     * @return Long
     * @throws Exceptions.OsoException
     */
    protected Long cacheInstance(Object instance, Long id) throws Exceptions.OsoException {
        if (id == null) {
            id = ffi.newId();
        }
        instances.put(id, instance);

        return id;
    }

    /**
     * Register a Java method call, wrapping the result in an enumeration if it
     * isn't already done.
     *
     * @param attrName   Name of the method/attribute.
     * @param args       Method arguments.
     * @param callId     Call ID under which to register the call.
     * @param instanceId ID of the Java instance on which to call the method.
     * @throws Exceptions.InvalidCallError
     */
    protected void registerCall(String attrName, List<Object> args, long callId, long instanceId)
            throws Exceptions.InvalidCallError {
        if (calls.containsKey(callId)) {
            return;
        }
        Object instance = instances.get(instanceId);
        // Get types of args to pass into `getMethod()`
        List<Class> argTypes = args.stream().map(a -> a.getClass()).collect(Collectors.toUnmodifiableList());
        Object result = null;
        Boolean isMethod = true;
        try {
            try {
                Method method = instance.getClass().getMethod(attrName, argTypes.toArray(new Class[argTypes.size()]));
                result = method.invoke(instance, args.toArray());
            } catch (NoSuchMethodException e) {
                isMethod = false;
            }
            if (!isMethod) {
                try {
                    Field field = instance.getClass().getField(attrName);
                    result = field.get(instance);
                } catch (NoSuchFieldException e) {
                    throw new Exceptions.InvalidCallError(attrName);
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
     * Get the next JSONified Polar result of a cached method call (enumeration).
     *
     * @param callId
     * @return JSONObject
     * @throws NoSuchElementException
     * @throws Exceptions.OsoException
     */
    protected JSONObject nextCallResult(long callId) throws NoSuchElementException, Exceptions.OsoException {
        return toPolarTerm(getCachedCall(callId).nextElement());
    }

    /**
     * Get a registered Java class.
     *
     * @param name
     * @return
     * @throws Exceptions.UnregisteredClassError
     */
    protected Class getRegisteredClass(String name) throws Exceptions.UnregisteredClassError {
        if (classes.containsKey(name)) {
            return classes.get(name);
        } else {
            throw new Exceptions.UnregisteredClassError(name);
        }
    }

    /**
     * Get a cached Java instance.
     *
     * @param instanceId
     * @return
     * @throws Exceptions.UnregisteredInstanceError
     */
    protected Object getCachedInstance(long instanceId) throws Exceptions.UnregisteredInstanceError {
        if (instances.containsKey(instanceId)) {
            return instances.get(instanceId);
        } else {
            throw new Exceptions.UnregisteredInstanceError(instanceId);
        }
    }

    /**
     * Determine if a Java instance has been cached.
     *
     * @param instanceId
     * @return
     */
    protected boolean hasInstance(long instanceId) {
        return instances.containsKey(instanceId);
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
    protected boolean subspecializer(long instanceId, String leftTag, String rightTag)
            throws Exceptions.UnregisteredClassError {
        Object instance = instances.get(instanceId);
        Class cls, leftClass, rightClass;
        cls = instance.getClass();
        leftClass = getRegisteredClass(leftTag);
        rightClass = getRegisteredClass(rightTag);

        boolean answer = false;
        if (leftClass.isInstance(instance) || rightClass.isInstance(instance)) {
            while (cls != null) {
                if (cls.equals(leftClass)) {
                    answer = true;
                    break;
                } else if (cls.equals(rightClass)) {
                    break;
                }
                cls = cls.getSuperclass();
            }
        }
        return answer;
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
    protected boolean isa(long instanceId, String classTag)
            throws Exceptions.UnregisteredClassError, Exceptions.UnregisteredInstanceError {
        Class cls = getRegisteredClass(classTag);
        Object instance = getCachedInstance(instanceId);
        return cls.isInstance(instance);
    }

    /*******************/
    /* PRIVATE METHODS */
    /*******************/

    /**
     * Load all queued files, flushing the {@code loadQueue}
     *
     * @throws Exceptions.OsoException
     */
    private void loadQueuedFiles() throws Exceptions.OsoException {
        for (String fname : loadQueue.keySet()) {
            loadStr(loadQueue.get(fname), fname);
        }
        loadQueue.clear();
    }

    /**
     * Confirm that all queued inline queries succeed.
     *
     * @throws Exceptions.OsoException           On failed query creation.
     * @throws Exceptions.InlineQueryFailedError On inline query failure.
     */
    private void checkInlineQueries() throws Exceptions.OsoException, Exceptions.InlineQueryFailedError {
        Ffi.Query nextQuery = ffi.nextInlineQuery();
        while (nextQuery != null) {
            if (!new Query(nextQuery, this).hasMoreElements()) {
                throw new Exceptions.InlineQueryFailedError();
            }
            nextQuery = ffi.nextInlineQuery();
        }
    }

    /**
     * Clear cached instances and calls.
     */
    private void clearQueryState() {
        instances.clear();
        calls.clear();
    }

    /**
     * Get cached Java method call result.
     *
     * @param callId
     * @return
     * @throws Exceptions.PolarRuntimeException
     */
    private Enumeration<Object> getCachedCall(long callId) throws Exceptions.PolarRuntimeException {
        if (calls.containsKey(callId)) {
            return calls.get(callId);
        } else {
            throw new Exceptions.PolarRuntimeException("Unregistered call ID: " + callId);
        }

    }

}
