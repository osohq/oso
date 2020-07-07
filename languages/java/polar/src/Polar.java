import jnr.ffi.Pointer;
import org.json.*;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.*;
import java.util.function.Function;

public class Polar {
    private Pointer polarPtr;
    private Ffi ffi;
    private Map<String, Class<Object>> classes;
    private Map<String, Function<Map, Object>> constructors;
    private Map<Long, Object> instances;
    private Map<String, String> loadQueue; // Map from filename -> file contents

    public Polar() {
        ffi = new Ffi();
        polarPtr = ffi.polarNew();
        classes = new HashMap<String, Class<Object>>();
        constructors = new HashMap<String, Function<Map, Object>>();
        instances = new HashMap<Long, Object>();
        loadQueue = new HashMap<String, String>();
    }

    @Override
    protected void finalize() {
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
     * @throws IOException If unable to open or read the file.
     */
    public void loadFile(String filename) throws IOException {
        Optional<String> ext = Optional.ofNullable(filename).filter(f -> f.contains("."))
                .map(f -> f.substring(filename.lastIndexOf(".") + 1));

        // check file extension
        if (!ext.isPresent() || !ext.get().equals("polar")) {
            throw new Error("Incorrect Polar file extension");
        }

        // add file to queue
        loadQueue.put(filename, new String(Files.readAllBytes(Paths.get(filename))));
    }

    /**
     * Load all queued files, flushing the {@code loadQueue}
     */
    public void loadQueuedFiles() {
        for (String fname : loadQueue.keySet()) {
            loadStr(loadQueue.get(fname), fname);
        }
        loadQueue.clear();
    }

    /**
     * Clear the KB, but maintain all registered classes and calls
     */
    public void clear() {
        // clear all Queued files
        loadQueue.clear();

        // Replace Polar instance
        ffi.polarFree(polarPtr);
        polarPtr = ffi.polarNew();
    }

    private void clearQueryState() {
        instances.clear();
        // TODO: clear calls
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     */
    public void loadStr(String str, String filename) {
        ffi.polarLoad(polarPtr, str, filename);
        checkInlineQueries();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     */
    public void loadStr(String str) {
        ffi.polarLoad(polarPtr, str, null);
        checkInlineQueries();

    }

    /**
     * Confirm that all queued inline queries succeed.
     *
     * @throws Error On inline query failure.
     */
    private void checkInlineQueries() {
        Pointer nextQuery = ffi.polarNextInlineQuery(polarPtr);
        while (nextQuery != null) {
            if (!new Query(nextQuery).hasMoreElements()) {
                throw new Error("Inline query failed");
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
    public Query queryStr(String queryStr) {
        loadQueuedFiles();
        return new Query(ffi.polarNewQuery(polarPtr, queryStr));
    }

    /**
     * Start the Polar REPL.
     */
    public void repl() {
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
     * @throws Error if class has already been registered.
     */
    public void registerClass(Class cls, Function<Map, Object> fromPolar) throws Error {
        if (classes.containsKey(cls.getName())) {
            throw new Error("A class named " + cls.getName() + " has already been registered.");
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
     * @throws Error if a class has already been registered with the given alias.
     */
    public void registerClass(Class cls, String alias, Function<Map, Object> fromPolar) throws Error {
        if (classes.containsKey(alias)) {
            throw new Error("A class named " + alias + " has already been registered.");
        }
        classes.put(alias, cls);
        constructors.put(alias, fromPolar);
    }

    /**
     * Make an instance of a Java class from a {@code Map<String, Object>} of
     * fields.
     *
     * @param cls
     * @param fields
     * @param id
     * @return Object
     */
    public Object makeInstance(String cls_name, Map fields, Long id) {
        Function<Map, Object> constructor = constructors.get(cls_name);
        Object instance;
        if (constructor != null) {
            instance = constructors.get(cls_name).apply(fields);
        } else {
            // TODO: default constructor
            throw new Error("unimplemented");
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
    private Long cacheInstance(Object instance, Long id) {
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
    public Object toJava(JSONObject term) {
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
                    throw new Error("Unregistered instance");
                }
            case "Call":
                // Predicate.new(value['name'], args: value['args'].map { |a| to_ruby(a) })
                throw new Error("Unimplemented Polar Type");
            default:
                throw new Error("Unexpected Polar Type");
        }
    }

    /**
     * Convert Java Objects to Polar (JSON) terms.
     *
     * @param value Java Object to be converted to Polar.
     * @return JSONObject Polar term of form: {@code {"id": _, "offset": _, "value":
     *         _}}.
     */
    public JSONObject toPolarTerm(Object value) {
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

        } else {
            jVal.put("ExternalInstance", new JSONObject().put("instance_id", cacheInstance(value, null)));
        }
        // TODO: Predicate, Variable, Symbol
        // when value.instance_of?(Predicate)
        // { 'Call' => { 'name' => value.name, 'args' => value.args.map { |el|
        // to_polar_term(el) } } }
        // when value.instance_of?(Variable)
        // # This is supported so that we can query for unbound variables
        // { 'Symbol' => value }

        // Build Polar term
        JSONObject term = new JSONObject();
        term.put("id", 0);
        term.put("offset", 0);
        term.put("value", jVal);
        return term;
    }

    public class Query implements Enumeration<HashMap<String, Object>> {
        private HashMap<String, Object> next;
        private Pointer queryPtr;

        /**
         * Construct a new Query object.
         *
         * @param queryPtr Pointer to the FFI query instance.
         */
        public Query(Pointer queryPtr) {
            this.queryPtr = queryPtr;
            next = nextResult();
        }

        @Override
        protected void finalize() {
            ffi.queryFree(queryPtr);
        }

        @Override
        public boolean hasMoreElements() {
            return next != null;
        }

        @Override
        public HashMap<String, Object> nextElement() {
            HashMap<String, Object> ret = next;
            next = nextResult();
            return ret;
        }

        public List<HashMap<String, Object>> results() {
            return Collections.list(this);
        }

        /**
         * Generate the next Query result.
         *
         * @return
         */
        private HashMap<String, Object> nextResult() {
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
                            throw new Error("Duplicate instance registration.");
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
                    default:
                        throw new Error("Unimplemented event type: " + kind);
                }
            }

        }

    }
}
