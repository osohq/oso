import jnr.ffi.Pointer;
import org.json.*;
import java.util.*;

public class Polar {
    private Pointer polar_ptr;
    private Ffi ffi_instance;

    public Polar() {
        ffi_instance = new Ffi();
        polar_ptr = ffi_instance.polar_new();
    }

    // Load a Polar string into the KB (with filename).
    public void load_str(String str, String filename) {
        ffi_instance.polar_load(polar_ptr, str, filename);
    }

    // Load a Polar string into the KB (without filename).
    public void load_str(String str) {
        ffi_instance.polar_load(polar_ptr, str, null);
    }

    // Query for a Polar string
    public Enumeration<HashMap<String, Object>> query_str(String query_str) {
        Query query = new Query(ffi_instance.polar_new_query(polar_ptr, query_str));
        return query.results;
    }

    // Free the Polar FFI object
    public void free() {
        ffi_instance.polar_free(polar_ptr);
    }

    // Start the Polar REPL
    public void repl() {
    }

    // Turn a Polar term passed across the FFI boundary into a Ruby value.
    public Object to_java(JSONObject data) {
        JSONObject value = data.getJSONObject("value");
        String tag = value.keys().next();
        switch (tag) {
            case "String":
                return value.getString(tag);
            case "Boolean":
                return value.getBoolean(tag);
            case "Number":
                return value.getJSONObject(tag).getInt("Integer");
            case "List":
                JSONArray jArray = value.getJSONArray(tag);
                ArrayList<Object> resArray = new ArrayList<Object>();
                for (int i = 0; i < jArray.length(); i++) {
                    resArray.add(to_java(jArray.getJSONObject(i)));
                }
                return resArray;
            case "Dictionary":
                JSONObject jMap = value.getJSONObject(tag).getJSONObject("fields");
                HashMap<String, Object> resMap = new HashMap<String, Object>();
                for (String key : jMap.keySet()) {
                    resMap.put(key, to_java(jMap.getJSONObject(key)));

                }
                return resMap;
            case "ExternalInstance":
                // get_instance(value['instance_id'])
                throw new Error("Unimplemented Polar Type");
            case "Call":
                // Predicate.new(value['name'], args: value['args'].map { |a| to_ruby(a) })
                throw new Error("Unimplemented Polar Type");
            default:
                throw new Error("Unexpected Polar Type");
        }
    }

    public class Query {
        private Pointer query_ptr;
        public Result results;

        public Query(Pointer query_ptr) {
            this.query_ptr = query_ptr;
            results = new Result();
        }

        // Query Results are Enumerations of Strings
        public class Result implements Enumeration<HashMap<String, Object>> {
            private HashMap<String, Object> next;

            public Result() {
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
                    String event_str = ffi_instance.polar_next_query_event(query_ptr);
                    String kind;
                    JSONObject data;
                    try {
                        JSONObject event = new JSONObject(event_str);
                        kind = event.keys().next();
                        data = event.getJSONObject(kind);
                    } catch (JSONException e) {
                        // TODO: this sucks, we should have a consistent serialization format
                        kind = event_str.replace("\"", "");
                        data = null;
                    }

                    switch (kind) {
                        case "Done":
                            return null;
                        case "Result":
                            HashMap<String, Object> results = new HashMap<String, Object>();
                            JSONObject bindings = data.getJSONObject("bindings");

                            for (String key : bindings.keySet()) {
                                Object val = to_java(bindings.getJSONObject(key));
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

    public static void main(String[] args) {
        Polar p = new Polar();
        p.load_str("f(1);");
        Enumeration<HashMap<String, Object>> results = p.query_str("f(x)");
        while (results.hasMoreElements()) {
            System.out.println(results.nextElement());
        }
        p.free();
    }

}
