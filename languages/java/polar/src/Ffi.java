import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;
import com.google.gson.JsonParser;

public class Ffi {
    private PolarLib polar_lib;

    protected static interface PolarLib {
        int polar_debug_command(Pointer query_ptr, String value);

        int polar_free(Pointer polar);

        String polar_get_error();

        long polar_get_external_id(Pointer polar_ptr);

        int polar_load(Pointer polar_ptr, String src, String filename);

        Pointer polar_new();

        Pointer polar_new_query(Pointer polar_ptr, String query_str);

        Pointer polar_new_query_from_term(Pointer polar_ptr, String query_term);

        Pointer polar_next_inline_query(Pointer polar_ptr);

        String polar_next_query_event(Pointer query_ptr);

        Pointer polar_query_from_repl(Pointer polar_ptr);

        int polar_question_result(Pointer query_ptr, long call_id, int result);

        int query_free(Pointer query);

        int string_free(Pointer s);

    }

    public Ffi() {
        polar_lib = LibraryLoader.create(PolarLib.class).load("lib/libpolar.dylib");
    }

    public int polar_free(Pointer polar_ptr) throws PolarRuntimeException {
        return check_int_result(polar_lib.polar_free(polar_ptr));
    }

    public int polar_debug_command(Pointer query_ptr, String value) throws PolarRuntimeException {
        return check_int_result(polar_lib.polar_debug_command(query_ptr, value));
    }

    public long polar_get_external_id(Pointer polar_ptr) throws PolarRuntimeException {
        return check_long_result(polar_lib.polar_get_external_id(polar_ptr));
    }

    public int polar_load(Pointer polar_ptr, String src, String filename) throws PolarRuntimeException {
        return check_int_result(polar_lib.polar_load(polar_ptr, src, filename));
    }

    public Pointer polar_new() throws PolarRuntimeException {
        return check_ptr_result(polar_lib.polar_new());
    }

    public Pointer polar_new_query(Pointer polar_ptr, String query_str) throws PolarRuntimeException {
        return check_ptr_result(polar_lib.polar_new_query(polar_ptr, query_str));
    }

    public Pointer polar_new_query_from_term(Pointer polar_ptr, String query_term) throws PolarRuntimeException {
        return check_ptr_result(polar_lib.polar_new_query_from_term(polar_ptr, query_term));
    }

    public Pointer polar_next_inline_query(Pointer polar_ptr) throws PolarRuntimeException {
        return check_ptr_result(polar_lib.polar_next_inline_query(polar_ptr));
    }

    public String polar_next_query_event(Pointer query_ptr) throws PolarRuntimeException {
        return check_str_result(polar_lib.polar_next_query_event(query_ptr));
    }

    public Pointer polar_query_from_repl(Pointer polar_ptr) throws PolarRuntimeException {
        return check_ptr_result(polar_lib.polar_query_from_repl(polar_ptr));
    }

    public int polar_question_result(Pointer query_ptr, long call_id, int result) throws PolarRuntimeException {
        return check_int_result(polar_lib.polar_question_result(query_ptr, call_id, result));
    }

    public int query_free(Pointer query) throws PolarRuntimeException {
        return check_int_result(polar_lib.query_free(query));
    }

    public int string_free(Pointer s) throws PolarRuntimeException {
        return check_int_result(polar_lib.string_free(s));
    }

    // Error handling
    private class PolarRuntimeException extends Error {
        PolarRuntimeException(String s) {
            super(s);
        }
    }

    private PolarRuntimeException get_error() {
        return new PolarRuntimeException(polar_lib.polar_get_error());
    }

    private int check_int_result(int i) throws PolarRuntimeException {
        if (i == 0) {
            throw get_error();
        } else {
            return i;
        }
    }

    private long check_long_result(long i) throws PolarRuntimeException {
        if (i == 0) {
            throw get_error();
        } else {
            return i;
        }
    }

    private Pointer check_ptr_result(Pointer p) throws PolarRuntimeException {
        if (p == null) {
            throw get_error();
        } else {
            return p;
        }
    }

    private String check_str_result(String s) throws PolarRuntimeException {
        if (s == null) {
            throw get_error();
        } else {
            return s;
        }

    }

    public static void main(String[] args) {
        Ffi ffi = new Ffi();
        try {
            Pointer p = ffi.polar_new();
        } catch (Exception e) {
            System.out.println(e);
        }
    }

}
