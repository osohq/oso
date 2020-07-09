import org.json.JSONObject;

import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;

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

        int polar_call_result(Pointer query_ptr, long call_id, String value);

        int query_free(Pointer query);

        int string_free(Pointer s);

    }

    public Ffi() {
        polar_lib = LibraryLoader.create(PolarLib.class).load("lib/libpolar.dylib");
    }

    public int polarFree(Pointer polar_ptr) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.polar_free(polar_ptr));
    }

    public int polarDebugCommand(Pointer query_ptr, String value) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.polar_debug_command(query_ptr, value));
    }

    public long polarGetExternalId(Pointer polar_ptr) throws Exceptions.OsoException {
        return checkLongResult(polar_lib.polar_get_external_id(polar_ptr));
    }

    public int polarLoad(Pointer polar_ptr, String src, String filename) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.polar_load(polar_ptr, src, filename));
    }

    public Pointer polarNew() throws Exceptions.OsoException {
        return checkPtrResult(polar_lib.polar_new());
    }

    public Pointer polarNewQuery(Pointer polar_ptr, String query_str) throws Exceptions.OsoException {
        return checkPtrResult(polar_lib.polar_new_query(polar_ptr, query_str));
    }

    public Pointer polarNewQueryFromTerm(Pointer polar_ptr, String query_term) throws Exceptions.OsoException {
        return checkPtrResult(polar_lib.polar_new_query_from_term(polar_ptr, query_term));
    }

    public Pointer polarNextInlineQuery(Pointer polar_ptr) throws Exceptions.OsoException {
        // Don't check result here because the returned Pointer is null to indicate
        // termination
        return polar_lib.polar_next_inline_query(polar_ptr);
    }

    public String polarNextQueryEvent(Pointer query_ptr) throws Exceptions.OsoException {
        return checkStrResult(polar_lib.polar_next_query_event(query_ptr));
    }

    public Pointer polarQueryFromRepl(Pointer polar_ptr) throws Exceptions.OsoException {
        return checkPtrResult(polar_lib.polar_query_from_repl(polar_ptr));
    }

    public int polarQuestionResult(Pointer query_ptr, long call_id, int result) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.polar_question_result(query_ptr, call_id, result));
    }

    public int polarCallResult(Pointer query_ptr, long call_id, String value) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.polar_call_result(query_ptr, call_id, value));
    }

    public int queryFree(Pointer query) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.query_free(query));
    }

    public int stringFree(Pointer s) throws Exceptions.OsoException {
        return checkIntResult(polar_lib.string_free(s));
    }

    private Exceptions.OsoException getError() {
        return Exceptions.getJavaError(polar_lib.polar_get_error());
    }

    private int checkIntResult(int i) throws Exceptions.OsoException {
        if (i == 0) {
            throw getError();
        } else {
            return i;
        }
    }

    private long checkLongResult(long i) throws Exceptions.OsoException {
        if (i == 0) {
            throw getError();
        } else {
            return i;
        }
    }

    private Pointer checkPtrResult(Pointer p) throws Exceptions.OsoException {
        if (p == null) {
            throw getError();
        } else {
            return p;
        }
    }

    private String checkStrResult(String s) throws Exceptions.OsoException {
        if (s == null) {
            throw getError();
        } else {
            return s;
        }

    }

    public static void main(String[] args) {
        Ffi ffi = new Ffi();
        try {
            Pointer p = ffi.polarNew();
            ffi.polarFree(p);
        } catch (Exception e) {
            System.out.println(e);
        }
    }

}
