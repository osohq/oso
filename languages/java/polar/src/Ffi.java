import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;

public class Ffi {
    private PolarLib polarLib;

    protected class PolarPtr {
        private Pointer ptr;

        protected PolarPtr(Pointer ptr) {
            this.ptr = ptr;
        }

        protected Pointer get() {
            return ptr;
        }

        @Override
        protected void finalize() {
            polarLib.polar_free(ptr);
        }
    }

    protected class QueryPtr {
        private Pointer ptr;

        protected QueryPtr(Pointer ptr) {
            this.ptr = ptr;
        }

        protected Pointer get() {
            return ptr;
        }

        protected int polarQuestionResult(long call_id, int result) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_question_result(ptr, call_id, result));
        }

        protected int polarCallResult(long call_id, String value) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_call_result(ptr, call_id, value));
        }

        protected String polarNextQueryEvent() throws Exceptions.OsoException {
            return checkResult(polarLib.polar_next_query_event(ptr));
        }

        @Override
        protected void finalize() {
            polarLib.polar_free(ptr);
        }

    }

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

    protected Ffi() {
        polarLib = LibraryLoader.create(PolarLib.class).load("lib/libpolar.dylib");
    }

    protected int polarDebugCommand(QueryPtr queryPtr, String value) throws Exceptions.OsoException {
        return checkResult(polarLib.polar_debug_command(queryPtr.get(), value));
    }

    protected long polarGetExternalId(PolarPtr polarPtr) throws Exceptions.OsoException {
        return checkResult(polarLib.polar_get_external_id(polarPtr.get()));
    }

    protected int polarLoad(PolarPtr polarPtr, String src, String filename) throws Exceptions.OsoException {
        return checkResult(polarLib.polar_load(polarPtr.get(), src, filename));
    }

    protected PolarPtr polarNew() throws Exceptions.OsoException {
        return new PolarPtr(checkResult(polarLib.polar_new()));
    }

    protected QueryPtr polarNewQuery(PolarPtr polarPtr, String queryStr) throws Exceptions.OsoException {
        return new QueryPtr(checkResult(polarLib.polar_new_query(polarPtr.get(), queryStr)));
    }

    protected QueryPtr polarNewQueryFromTerm(PolarPtr polarPtr, String queryTerm) throws Exceptions.OsoException {
        return new QueryPtr(checkResult(polarLib.polar_new_query_from_term(polarPtr.get(), queryTerm)));
    }

    protected QueryPtr polarNextInlineQuery(PolarPtr polarPtr) throws Exceptions.OsoException {
        // Don't check result here because the returned Pointer is null to indicate
        // termination
        Pointer p = polarLib.polar_next_inline_query(polarPtr.get());
        if (p == null) {
            return null;
        } else {
            return new QueryPtr(p);
        }
    }

    protected QueryPtr polarQueryFromRepl(PolarPtr polarPtr) throws Exceptions.OsoException {
        return new QueryPtr(checkResult(polarLib.polar_query_from_repl(polarPtr.get())));
    }

    protected int stringFree(Pointer s) throws Exceptions.OsoException {
        return checkResult(polarLib.string_free(s));
    }

    private Exceptions.OsoException getError() {
        return Exceptions.getJavaError(polarLib.polar_get_error());
    }

    private int checkResult(int i) throws Exceptions.OsoException {
        if (i == 0) {
            throw getError();
        } else {
            return i;
        }
    }

    private long checkResult(long i) throws Exceptions.OsoException {
        if (i == 0) {
            throw getError();
        } else {
            return i;
        }
    }

    private Pointer checkResult(Pointer p) throws Exceptions.OsoException {
        if (p == null) {
            throw getError();
        } else {
            return p;
        }
    }

    private String checkResult(String s) throws Exceptions.OsoException {
        if (s == null) {
            throw getError();
        } else {
            return s;
        }

    }
}
