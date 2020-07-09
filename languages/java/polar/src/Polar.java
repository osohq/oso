import jnr.ffi.Pointer;
import org.json.*;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.*;
import java.util.function.Function;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Field;

public class Polar {
    private Pointer polarPtr;
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
     * @throws Exceptions.OsoException
     */
    @Override
    protected void finalize() throws Exceptions.OsoException {
        // Free the Polar FFI object
        ffi.polarFree(polarPtr);
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
        ffi.polarFree(polarPtr);
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
        Pointer nextQuery = ffi.polarNextInlineQuery(polarPtr);
        while (nextQuery != null) {
            if (!new Query(nextQuery).hasMoreElements()) {
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
        return new Query(ffi.polarNewQuery(polarPtr, queryStr));
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
        return new Query(ffi.polarNewQueryFromTerm(polarPtr, pred));
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
            Query query = new Query(ffi.polarQueryFromRepl(polarPtr));
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
     * @param alias     name to register the class under, which is how the class is
     *                  accessed from Polar.
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @throws Exceptions.DuplicateClassAliasError if a class has already been
     *                                             registered with the given alias.
     */
    public void registerClass(Class cls, String alias, Function<Map, Object> fromPolar)
            throws Exceptions.DuplicateClassAliasError {
        if (classes.containsKey(alias)) {
            throw new Exceptions.DuplicateClassAliasError(alias, classes.get(alias).getName(), cls.getName());
        }
        classes.put(alias, cls);
        constructors.put(alias, fromPolar);
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
        Class[] argTypes = new Class[args.size()];
        for (int i = 0; i < args.size(); i++) {
            argTypes[i] = args.get(i).getClass();
        }
        Object result = null;
        Boolean isMethod = true;
        try {
            try {
                Method method = instance.getClass().getMethod(attrName, argTypes);
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
                JSONArray jArray = value.getJSONArray(tag);
                ArrayList<Object> resArray = new ArrayList<Object>();
                for (int i = 0; i < jArray.length(); i++) {
                    resArray.add(toJava(jArray.getJSONObject(i)));
                }
                return resArray;
            case "Dictionary":
                JSONObject jMap = value.getJSONObject(tag).getJSONObject("fields");
                HashMap<String, Object> resMap = new HashMap<String, Object>();
                for (String key : jMap.keySet()) {
                    resMap.put(key, toJava(jMap.getJSONObject(key)));

                }
                return resMap;
            case "ExternalInstance":
                Long id = value.getJSONObject(tag).getLong("instance_id");
                if (instances.containsKey(id)) {
                    return instances.get(id);
                } else {
                    throw new Exceptions.UnregisteredInstanceError(id);
                }
            case "Call":
                // TODO: implement helper to go from Polar list -> Java list and back
                JSONArray jArgs = value.getJSONObject(tag).getJSONArray("args");
                ArrayList<Object> args = new ArrayList<Object>();
                for (int i = 0; i < jArgs.length(); i++) {
                    args.add(toJava(jArgs.getJSONObject(i)));
                }
                return new Predicate(value.getJSONObject(tag).getString("name"), args);
            default:
                throw new Exceptions.UnexpectedPolarTypeError(tag);
        }
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

    /**
     *
     * @param name
     * @return
     * @throws Exceptions.UnregisteredClassError
     */
    private Class getPolarClass(String name) throws Exceptions.UnregisteredClassError {
        if (classes.containsKey(name)) {
            return classes.get(name);
        } else {
            throw new Exceptions.UnregisteredClassError(name);
        }
    }

    public class Query implements Enumeration<HashMap<String, Object>> {
        private HashMap<String, Object> next;
        private Pointer queryPtr;

        /**
         * Construct a new Query object.
         *
         * @param queryPtr Pointer to the FFI query instance.
         */
        public Query(Pointer queryPtr) throws Exceptions.OsoException {
            this.queryPtr = queryPtr;
            next = nextResult();
        }

        @Override
        protected void finalize() throws Exceptions.OsoException {
            ffi.queryFree(queryPtr);
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

        public List<HashMap<String, Object>> results() {
            return Collections.list(this);
        }

        /**
         * Generate the next Query result.
         *
         * @return
         * @throws Exceptions.OsoException
         */
        private HashMap<String, Object> nextResult() throws Exceptions.OsoException {
            while (true) {
                String eventStr = ffi.polarNextQueryEvent(queryPtr);
                String kind;
                JSONObject data;
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
                        HashMap<String, Object> results = new HashMap<String, Object>();
                        JSONObject bindings = data.getJSONObject("bindings");

                        for (String key : bindings.keySet()) {
                            Object val = toJava(bindings.getJSONObject(key));
                            results.put(key, val);
                        }
                        return results;
                    case "MakeExternal":
                        Long id = data.getLong("instance_id");
                        if (instances.containsKey(id)) {
                            throw new Exceptions.DuplicateInstanceRegistrationError(id);
                        }
                        String clsName = data.getJSONObject("instance").getString("tag");
                        JSONObject jFields = data.getJSONObject("instance").getJSONObject("fields")
                                .getJSONObject("fields");
                        Map<String, Object> fields = new HashMap<String, Object>();
                        for (String k : jFields.keySet()) {
                            fields.put(k, toJava(jFields.getJSONObject(k)));
                        }
                        makeInstance(clsName, fields, id);
                        break;
                    case "ExternalCall":
                        long callId = data.getLong("call_id");
                        long instanceId = data.getLong("instance_id");
                        String attrName = data.getString("attribute");
                        JSONArray jArgs = data.getJSONArray("args");
                        List<Object> args = new ArrayList<Object>();
                        for (int i = 0; i < jArgs.length(); i++) {
                            args.add(toJava(jArgs.getJSONObject(i)));
                        }
                        registerCall(attrName, args, callId, instanceId);
                        String result;
                        try {
                            result = nextCallResult(callId).toString();
                        } catch (NoSuchElementException e) {
                            result = null;
                        }
                        ffi.polarCallResult(queryPtr, callId, result);
                        break;
                    case "ExternalIsa":
                        instanceId = data.getLong("instance_id");
                        callId = data.getLong("call_id");
                        String classTag = data.getString("class_tag");
                        Class cls = getPolarClass(classTag);
                        Object instance = instances.get(instanceId);
                        int answer = classes.containsKey(classTag) && cls.isInstance(instance) ? 1 : 0;
                        ffi.polarQuestionResult(queryPtr, callId, answer);
                        break;
                    case "ExternalIsSubSpecializer":
                        instanceId = data.getLong("instance_id");
                        callId = data.getLong("call_id");
                        instance = instances.get(instanceId);
                        cls = instance.getClass();
                        Class leftClass = getPolarClass(data.getString("left_class_tag"));
                        Class rightClass = getPolarClass(data.getString("right_class_tag"));

                        answer = 0;
                        if (leftClass.isInstance(instance) || rightClass.isInstance(instance)) {
                            while (cls != null) {
                                if (cls.equals(leftClass)) {
                                    answer = 1;
                                    break;
                                } else if (cls.equals(rightClass)) {
                                    break;
                                }
                                cls = cls.getSuperclass();
                            }
                        }
                        ffi.polarQuestionResult(queryPtr, callId, answer);
                        break;
                    default:
                        throw new Exceptions.PolarRuntimeException("Unhandled event type: " + kind);
                }
            }

        }

    }

    public static class Predicate {
        public String name;
        public List<Object> args;

        public Predicate(String name, List<Object> args) {
            this.name = name;
            this.args = args;
        }

        @Override
        public boolean equals(Object obj) {
            if (!(obj instanceof Predicate)) {
                return false;
            }
            if (((Predicate) obj).name.equals(this.name) && ((Predicate) obj).args.equals(this.args)) {
                return true;
            } else {
                return false;
            }
        }
    }

    public static class Variable {
        public String name;

        public Variable(String name) {
            this.name = name;
        }

        @Override
        public String toString() {
            return name;
        }
    }
}
