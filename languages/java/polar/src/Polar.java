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
        polarPtr = ffi.polar_new();
        classes = new HashMap<String, Class<Object>>();
        constructors = new HashMap<String, Function<Map, Object>>();
        instances = new HashMap<Long, Object>();
        loadQueue = new HashMap<String, String>();
    }

    @Override
    protected void finalize() {
        // Free the Polar FFI object
        ffi.polar_free(polarPtr);
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
        if (!ext.isPresent() || ext.get() != "polar") {
            throw new Error("Incorrect Polar file extension");
        }

        // add file to queue
        loadQueue.put(filename, new String(Files.readAllBytes(Paths.get(filename))));
    }

    public void loadQueuedFiles() {
        for (String fname : loadQueue.keySet()) {
            loadStr(loadQueue.get(fname), fname);
        }
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     */
    public void loadStr(String str, String filename) {
        ffi.polar_load(polarPtr, str, filename);
        checkInlineQueries();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     */
    public void loadStr(String str) {
        ffi.polar_load(polarPtr, str, null);
        checkInlineQueries();

    }

    /**
     * Confirm that all queued inline queries succeed.
     *
     * @throws Error On inline query failure.
     */
    private void checkInlineQueries() {
        Pointer nextQuery = ffi.polar_next_inline_query(polarPtr);
        while (nextQuery != null) {
            if (!new Query(nextQuery).hasMoreElements()) {
                throw new Error("Inline query failed");
            }
            nextQuery = ffi.polar_next_inline_query(polarPtr);
        }
    }

    /**
     * Query for a Polar string.
     *
     * @param queryStr
     * @return
     */
    public Query queryStr(String queryStr) {
        return new Query(ffi.polar_new_query(polarPtr, queryStr));
    }

    // Start the Polar REPL
    public void repl() {
        // clear_query_state
        // load_queued_files
        while (true) {
            Query query = new Query(ffi.polar_query_from_repl(polarPtr));
            if (!query.hasMoreElements()) {
                System.out.println("False");
            } else {
                do {
                    System.out.println(query.nextElement());
                } while (query.hasMoreElements());
            }

        }
    }

    public void registerClass(Class cls, Function<Map, Object> fromPolar) {
        classes.put(cls.getName(), cls);
        constructors.put(cls.getName(), fromPolar);

    }

    public Object makeInstance(Class cls, Map fields, Long id) {
        Function<Map, Object> constructor = constructors.get(cls.getName());
        Object instance;
        if (constructor != null) {
            instance = constructors.get(cls.getName()).apply(fields);
        } else {
            // TODO: default constructor
            throw new Error("unimplemented");
        }
        cacheInstance(instance, id);
        return instance;
    }

    public Long cacheInstance(Object instance, Long id) {
        if (id == null) {
            id = ffi.polar_get_external_id(polarPtr);
        }
        instances.put(id, instance);

        return id;
    }

    // Turn a Polar term passed across the FFI boundary into a Ruby value.
    public Object toJava(JSONObject data) {
        JSONObject value = data.getJSONObject("value");
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

    // Query Results are Enumerations of Strings
    public class Query implements Enumeration<HashMap<String, Object>> {
        private HashMap<String, Object> next;
        private Pointer queryPtr;

        public Query(Pointer queryPtr) {
            this.queryPtr = queryPtr;
            next = nextResult();
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

        private HashMap<String, Object> nextResult() {
            while (true) {
                String eventStr = ffi.polar_next_query_event(queryPtr);
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
                    default:
                        throw new Error("Unimplemented event type: " + kind);
                }
            }

        }

    }
}
