package com.osohq.oso;

import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;

public class Ffi {
    private PolarLib polarLib;

    protected class Polar {
        private Pointer ptr;

        private Polar(Pointer ptr) {
            this.ptr = ptr;
        }

        protected Pointer get() {
            return ptr;
        }

        protected long newId() throws Exceptions.OsoException {
            return checkResult(polarLib.polar_get_external_id(ptr));
        }

        protected int loadStr(String src, String filename) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_load(ptr, src, filename));
        }

        protected Query newQueryFromStr(String queryStr) throws Exceptions.OsoException {
            return new Query(checkResult(polarLib.polar_new_query(ptr, queryStr)));
        }

        protected Query newQueryFromTerm(String queryTerm) throws Exceptions.OsoException {
            return new Query(checkResult(polarLib.polar_new_query_from_term(ptr, queryTerm)));
        }

        protected Query nextInlineQuery() throws Exceptions.OsoException {
            // Don't check result here because the returned Pointer is null to indicate
            // termination
            Pointer p = polarLib.polar_next_inline_query(ptr);
            if (p == null) {
                return null;
            } else {
                return new Query(p);
            }
        }

        protected Query newQueryFromRepl() throws Exceptions.OsoException {
            return new Query(checkResult(polarLib.polar_query_from_repl(ptr)));
        }

        @Override
        protected void finalize() {
            polarLib.polar_free(ptr);
        }
    }

    protected class Query {
        private Pointer ptr;

        private Query(Pointer ptr) {
            this.ptr = ptr;
        }

        protected Pointer get() {
            return ptr;
        }

        protected int questionResult(long call_id, int result) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_question_result(ptr, call_id, result));
        }

        protected int callResult(long call_id, String value) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_call_result(ptr, call_id, value));
        }

        protected QueryEvent nextEvent() throws Exceptions.OsoException {
            return new QueryEvent(checkResult(polarLib.polar_next_query_event(ptr)));
        }

        protected int debugCommand(String value) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_debug_command(ptr, value));
        }

        @Override
        protected void finalize() {
            polarLib.query_free(ptr);
        }

    }

    protected class QueryEvent {
        private Pointer ptr;

        private QueryEvent(Pointer ptr) {
            this.ptr = ptr;
        }

        public String get() {
            return ptr.getString(0);
        }

        @Override
        protected void finalize() {
            polarLib.string_free(ptr);
        }

    }

    protected class Error {
        private Pointer ptr;

        protected Error() {
            ptr = polarLib.polar_get_error();
        }

        private Exceptions.OsoException get() {
            return Exceptions.getJavaError(ptr.getString(0));
        }

        @Override
        protected void finalize() {
            polarLib.string_free(ptr);
        }

    }

    protected static interface PolarLib {
        int polar_debug_command(Pointer query_ptr, String value);

        int polar_free(Pointer polar);

        Pointer polar_get_error();

        long polar_get_external_id(Pointer polar_ptr);

        int polar_load(Pointer polar_ptr, String src, String filename);

        Pointer polar_new();

        Pointer polar_new_query(Pointer polar_ptr, String query_str);

        Pointer polar_new_query_from_term(Pointer polar_ptr, String query_term);

        Pointer polar_next_inline_query(Pointer polar_ptr);

        Pointer polar_next_query_event(Pointer query_ptr);

        Pointer polar_query_from_repl(Pointer polar_ptr);

        int polar_question_result(Pointer query_ptr, long call_id, int result);

        int polar_call_result(Pointer query_ptr, long call_id, String value);

        int query_free(Pointer query);

        int string_free(Pointer s);

    }

    protected Ffi() {
        polarLib = LibraryLoader.create(PolarLib.class).load("../../../target/debug/libpolar.dylib");
    }

    protected Polar polarNew() throws Exceptions.OsoException {
        return new Polar(checkResult(polarLib.polar_new()));
    }

    protected int stringFree(Pointer s) throws Exceptions.OsoException {
        return checkResult(polarLib.string_free(s));
    }

    private int checkResult(int i) throws Exceptions.OsoException {
        if (i == 0) {
            throw new Error().get();
        } else {
            return i;
        }
    }

    private long checkResult(long i) throws Exceptions.OsoException {
        if (i == 0) {
            throw new Error().get();
        } else {
            return i;
        }
    }

    private Pointer checkResult(Pointer p) throws Exceptions.OsoException {
        if (p == null) {
            throw new Error().get();
        } else {
            return p;
        }
    }
}
