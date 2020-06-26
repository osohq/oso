import jnr.ffi.Pointer;
import java.util.Enumeration;
import org.json.JSONObject;

public class Polar {
    private Pointer polar_ptr;
    private Ffi ffi_instance;

    public Polar() throws Exception {
        ffi_instance = new Ffi();
        polar_ptr = ffi_instance.polar_new();
    }

    // Load a Polar string into the KB (with filename).
    public void load_str(String str, String filename) throws Exception {
        ffi_instance.polar_load(polar_ptr, str, filename);
    }

    // Load a Polar string into the KB (without filename).
    public void load_str(String str) throws Exception {
        ffi_instance.polar_load(polar_ptr, str, null);
    }

    public void query_str(String query_str) throws Exception {
        Query query = new Query(ffi_instance.polar_new_query(polar_ptr, query_str));
    }

    public void free() throws Exception {
        ffi_instance.polar_free(polar_ptr);
    }

    private class Query {
        private Pointer query_ptr;

        public Query(Pointer query_ptr) {
            this.query_ptr = query_ptr;
        }

        public void run() {

        }

        private class Result implements Enumeration {

            @Override
            public boolean hasMoreElements() {
                return false;
            }

            @Override
            public Object nextElement() {
                while (true) {
                    JSONObject event = new JSONObject(ffi_instance.polar_next_query_event(query_ptr));

                }
            }

        }
    }

    public static void main(String[] args) {
        try {
            Polar p = new Polar();
            p.load_str("f(1);");
            p.free();
        } catch (Exception e) {
            System.out.println(e);

        }
    }

}
