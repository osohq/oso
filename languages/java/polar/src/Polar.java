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
    private Ffi.PolarPtr polarPtr;
    private Ffi ffi;
    private Map<String, Class<Object>> classes;
    private Map<String, Function<Map, Object>> constructors;
    private Map<Long, Object> instances;
    private Map<String, String> loadQueue; // Map from filename -> file contents
    private Map<Long, Enumeration<Object>> calls;

    public Polar() throws Exceptions.OsoException {
        ffi = new Ffi();
        polarPtr = ffi.polarNew();
        classes = new HashMap<String, Class<Object>>();
        constructors = new HashMap<String, Function<Map, Object>>();
        instances = new HashMap<Long, Object>();
        loadQueue = new HashMap<String, String>();
        calls = new HashMap<Long, Enumeration<Object>>();
    }

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
    public void loadFile(String filename) throws IOException, Exceptions.PolarFileExtensionError {
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
     * Load all queued files, flushing the {@code loadQueue}
     */
    public void loadQueuedFiles() throws Exceptions.OsoException {
        for (String fname : loadQueue.keySet()) {
            loadStr(loadQueue.get(fname), fname);
        }
        loadQueue.clear();
    }

    /**
     * Clear the KB, but maintain all registered classes and calls
     */
    public void clear() throws Exceptions.OsoException {
        // clear all Queued files
        loadQueue.clear();

        // Replace Polar instance
        polarPtr = ffi.polarNew();
    }

    private void clearQueryState() {
        instances.clear();
        calls.clear();
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     */
    public void loadStr(String str, String filename) throws Exceptions.OsoException {
        ffi.polarLoad(polarPtr, str, filename);
        checkInlineQueries();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     */
    public void loadStr(String str) throws Exceptions.OsoException {
        ffi.polarLoad(polarPtr, str, null);
        checkInlineQueries();

    }

    /**
     * Confirm that all queued inline queries succeed.
     *
     * @throws Exceptions.OsoException           On failed query creation.
     * @throws Exceptions.InlineQueryFailedError On inline query failure.
     */
    private void checkInlineQueries() throws Exceptions.OsoException, Exceptions.InlineQueryFailedError {
        Ffi.QueryPtr nextQuery = ffi.polarNextInlineQuery(polarPtr);
        while (nextQuery != null) {
            if (!new Query(nextQuery, this).hasMoreElements()) {
                throw new Exceptions.InlineQueryFailedError();
            }
            nextQuery = ffi.polarNextInlineQuery(polarPtr);
        }
    }

    /**
     * Query for a Polar string.
     *
     * @param queryStr
     * @return Query object.
     */
    public Query queryStr(String queryStr) throws Exceptions.OsoException {
        clearQueryState();
        loadQueuedFiles();
        return new Query(ffi.polarNewQuery(polarPtr, queryStr), this);
    }

    /**
     * Query for a Predicate.
     *
     * @param name
     * @param args
     * @return
     */
    public Query queryPred(String name, List<Object> args) throws Exceptions.OsoException {
        clearQueryState();
        loadQueuedFiles();
        String pred = toPolarTerm(new Predicate(name, args)).toString();
        return new Query(ffi.polarNewQueryFromTerm(polarPtr, pred), this);
    }

    /**
     * Start the Polar REPL.
     *
     * @throws Exceptions.OsoException
     */
    public void repl() throws Exceptions.OsoException {
        // clear_query_state
        loadQueuedFiles();
        while (true) {
            Query query = new Query(ffi.polarQueryFromRepl(polarPtr), this);
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
     * @param cls
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @throws Exceptions.DuplicateClassAliasError if class has already been
     *                                             registered.
     */
    public void registerClass(Class cls, Function<Map, Object> fromPolar) throws Exceptions.DuplicateClassAliasError {
        String name = cls.getName();
        if (classes.containsKey(name)) {
            throw new Exceptions.DuplicateClassAliasError(name, classes.get(name).getName(), name);
        }
        classes.put(cls.getName(), cls);
        constructors.put(cls.getName(), fromPolar);
    }

    /**
     *
     * Register a Java class with oso using an alias.
     *
     * @param cls
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @param alias     name to register the class under, which is how the class is
     *                  accessed from Polar.
     * @throws Exceptions.DuplicateClassAliasError if a class has already been
     *                                             registered with the given alias.
     */
    public void registerClass(Class cls, Function<Map, Object> fromPolar, String alias)
            throws Exceptions.DuplicateClassAliasError {
        if (classes.containsKey(alias)) {
            throw new Exceptions.DuplicateClassAliasError(alias, classes.get(alias).getName(), cls.getName());
        }
        classes.put(alias, cls);
        constructors.put(alias, fromPolar);
    }

    protected String nextQueryEvent(Ffi.QueryPtr queryPtr) throws Exceptions.OsoException {
        return ffi.polarNextQueryEvent(queryPtr);
    }

    /**
     *
     * @param attrName
     * @param args
     * @param callId
     * @param instanceId
     * @throws Exceptions.InvalidCallError
     */
    public void registerCall(String attrName, List<Object> args, long callId, long instanceId)
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
                    // do nothing, let result = null. This will cause query to fail.
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
     *
     * @param callId
     * @return
     * @throws NoSuchElementException
     * @throws Exceptions.OsoException
     */
    public JSONObject nextCallResult(long callId) throws NoSuchElementException, Exceptions.OsoException {
        return toPolarTerm(calls.get(callId).nextElement());
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
    public Object makeInstance(String clsName, Map fields, long id) throws Exceptions.OsoException {
        Function<Map, Object> constructor = constructors.get(clsName);
        Object instance;
        if (constructor != null) {
            instance = constructors.get(clsName).apply(fields);
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
     */
    public Long cacheInstance(Object instance, Long id) throws Exceptions.OsoException {
        if (id == null) {
            id = ffi.polarGetExternalId(polarPtr);
        }
        instances.put(id, instance);

        return id;
    }

    /**
     * Turn a Polar term passed across the FFI boundary into a Java Object.
     *
     * @param term JSONified Polar term of the form: {@code {"id": _, "offset": _,
     *             "value": _}}
     * @return Object
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
                return getCachedInstance(value.getJSONObject(tag).getLong("instance_id"));
            case "Call":
                List<Object> args = polarListToJava(value.getJSONObject(tag).getJSONArray("args"));
                return new Predicate(value.getJSONObject(tag).getString("name"), args);
            default:
                throw new Exceptions.UnexpectedPolarTypeError(tag);
        }
    }

    /**
     *
     * @param dict
     * @return
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
     *
     * @param list
     * @return
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
     * Convert Java Objects to Polar (JSON) terms.
     *
     * @param value Java Object to be converted to Polar.
     * @return JSONObject Polar term of form: {@code {"id": _, "offset": _, "value":
     *         _}}.
     */
    public JSONObject toPolarTerm(Object value) throws Exceptions.OsoException {
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
            ArrayList<JSONObject> list = new ArrayList<JSONObject>();
            for (Object el : (List<Object>) value) {
                list.add(toPolarTerm(el));
            }
            jVal.put("List", list);

        } else if (value instanceof Map) {
            Map<Object, Object> map = (Map<Object, Object>) value;
            HashMap<String, JSONObject> jMap = new HashMap<String, JSONObject>();
            for (Object key : map.keySet()) {
                JSONObject val = toPolarTerm(map.get(key));
                jMap.put(key.toString(), val);
            }
            jVal.put("Dictionary", new JSONObject().put("fields", jMap));

        } else if (value instanceof Predicate) {
            Predicate pred = (Predicate) value;
            ArrayList<JSONObject> args = new ArrayList<JSONObject>();
            for (Object el : pred.args) {
                args.add(toPolarTerm(el));
            }
            jVal.put("Call", new JSONObject(Map.of("name", pred.name, "args", args)));
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

    protected boolean subspecializer(long instanceId, String leftTag, String rightTag)
            throws Exceptions.UnregisteredClassError {
        Object instance = instances.get(instanceId);
        Class cls, leftClass, rightClass;
        cls = instance.getClass();
        leftClass = getPolarClass(leftTag);
        rightClass = getPolarClass(rightTag);

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

    protected boolean isa(long instanceId, String classTag)
            throws Exceptions.UnregisteredClassError, Exceptions.UnregisteredInstanceError {
        Class cls = getPolarClass(classTag);
        Object instance = getCachedInstance(instanceId);
        return cls.isInstance(instance);
    }

    /**
     *
     * @param name
     * @return
     * @throws Exceptions.UnregisteredClassError
     */
    protected Class getPolarClass(String name) throws Exceptions.UnregisteredClassError {
        if (classes.containsKey(name)) {
            return classes.get(name);
        } else {
            throw new Exceptions.UnregisteredClassError(name);
        }
    }

    protected Object getCachedInstance(long instanceId) throws Exceptions.UnregisteredInstanceError {
        if (instances.containsKey(instanceId)) {
            return instances.get(instanceId);
        } else {
            throw new Exceptions.UnregisteredInstanceError(instanceId);
        }
    }

    protected boolean hasInstance(long instanceId) {
        return instances.containsKey(instanceId);
    }

}
