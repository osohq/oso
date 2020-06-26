import jnr.ffi.Pointer;
import java.util.Enumeration;
import org.json.JSONObject;
import org.json.JSONException;

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

    public Enumeration<String> query_str(String query_str) {
        Query query = new Query(ffi_instance.polar_new_query(polar_ptr, query_str));
        return query.results;
    }

    public void free() {
        ffi_instance.polar_free(polar_ptr);
    }

    private class Query {
        private Pointer query_ptr;
        public Result results;

        public Query(Pointer query_ptr) {
            this.query_ptr = query_ptr;
            results = new Result();
        }

        private class Result implements Enumeration<String> {
            private boolean has_next;

            public Result() {
                has_next = true;
            }

            @Override
            public boolean hasMoreElements() {
                return has_next;
            }

            @Override
            public String nextElement() {
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
                            has_next = false;
                            return null;
                        case "Result":
                            return data.getJSONObject("bindings").toString();
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
        Enumeration<String> results = p.query_str("f(x)");
        String bindings = results.nextElement();
        do {
            System.out.println(bindings);
            bindings = results.nextElement();
        } while (bindings != null);
        p.free();
    }

}
