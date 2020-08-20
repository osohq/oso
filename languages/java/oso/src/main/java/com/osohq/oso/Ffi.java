package com.osohq.oso;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;

import org.json.JSONObject;
import org.json.JSONException;

public class Ffi {
    // singleton variable
    private static Ffi ffi = null;

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
            int result = checkResult(polarLib.polar_load(ptr, src, filename));
            processMessages();
            return result;
        }

        protected Query newQueryFromStr(String queryStr) throws Exceptions.OsoException {
            Query query = new Query(checkResult(polarLib.polar_new_query(ptr, queryStr, 0)));
            processMessages();
            return query;
        }

        protected Query newQueryFromTerm(String queryTerm) throws Exceptions.OsoException {
            Query query = new Query(checkResult(polarLib.polar_new_query_from_term(ptr, queryTerm, 0)));
            processMessages();
            return query;
        }

        protected Query nextInlineQuery() throws Exceptions.OsoException {
            // Don't check result here because the returned Pointer is null to indicate
            // termination
            Pointer p = polarLib.polar_next_inline_query(ptr, 0);
            processMessages();
            if (p == null) {
                return null;
            } else {
                return new Query(p);
            }
        }

        protected int registerConstant(String name, String value) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_register_constant(ptr, name, value));
        }

        protected Pointer nextMessage() throws Exceptions.OsoException {
            return polarLib.polar_next_polar_message(ptr);
        }

        private void processMessages() throws Exceptions.OsoException {
            while (true) {
                Pointer msgPtr = nextMessage();
                if (msgPtr == null) {
                    break;
                }
                processMessage(msgPtr);
            }
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

        protected int applicationError(String message) throws Exceptions.OsoException {
            return checkResult(polarLib.polar_application_error(ptr, message));
        }

        protected QueryEvent nextEvent() throws Exceptions.OsoException {
            QueryEvent event = new QueryEvent(checkResult(polarLib.polar_next_query_event(ptr)));
            processMessages();
            return event;
        }

        protected int debugCommand(String value) throws Exceptions.OsoException {
            int result = checkResult(polarLib.polar_debug_command(ptr, value));
            processMessages();
            return result;
        }

        protected Pointer nextMessage() throws Exceptions.OsoException {
            return polarLib.polar_next_query_message(ptr);
        }

        private void processMessages() throws Exceptions.OsoException {
            while (true) {
                Pointer msgPtr = nextMessage();
                if (msgPtr == null) {
                    break;
                }
                processMessage(msgPtr);
            }
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

        Pointer polar_new_query(Pointer polar_ptr, String query_str, int trace);

        Pointer polar_new_query_from_term(Pointer polar_ptr, String query_term, int trace);

        Pointer polar_next_inline_query(Pointer polar_ptr, int trace);

        Pointer polar_next_query_event(Pointer query_ptr);

        Pointer polar_query_from_repl(Pointer polar_ptr);

        int polar_question_result(Pointer query_ptr, long call_id, int result);

        int polar_call_result(Pointer query_ptr, long call_id, String value);

        int polar_application_error(Pointer query_ptr, String message);

        int query_free(Pointer query);

        int string_free(Pointer s);

        int polar_register_constant(Pointer polar_ptr, String name, String value);

        Pointer polar_next_polar_message(Pointer polar_ptr);

        Pointer polar_next_query_message(Pointer query_ptr);

    }

    protected Ffi() {
        String platform = System.getProperty("os.name").toLowerCase();
        String path = null;
        String prefix = null;
        String suffix = null;

        if (platform.contains("win")) {
            path = "win/polar.dll";
            prefix = "polar";
            suffix = ".dll";
        } else if (platform.contains("mac")) {
            path = "macos/libpolar.dylib";
            prefix = "libpolar";
            suffix = ".dylib";
        } else {
            path = "linux/libpolar.so";
            prefix = "libpolar";
            suffix = ".so";
        }
        try {
            InputStream input = getClass().getClassLoader().getResourceAsStream(path);
            File file = File.createTempFile(prefix, suffix);
            OutputStream out = new FileOutputStream(file);
            int read;
            byte[] bytes = new byte[1024];
            while ((read = input.read(bytes)) != -1) {
                out.write(bytes, 0, read);
            }
            out.close();
            file.deleteOnExit();
            polarLib = LibraryLoader.create(PolarLib.class).load(file.getAbsolutePath());
        } catch (IOException e) {

        }
    }

    protected static Ffi get() {
        if (ffi == null) {
            ffi = new Ffi();
        }
        return ffi;
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

    private void processMessage(Pointer msgPtr) throws Exceptions.OsoException {
        if (msgPtr == null) {
            return;
        }
        String msgStr = msgPtr.getString(0);
        stringFree(msgPtr);
        try {
            JSONObject message = new JSONObject(msgStr);
            String kind = message.getString("kind");
            String msg = message.getString("msg");
            if (kind.equals("Print")) {
                System.out.println(msg);
            } else if (kind.equals("Warning")) {
                System.err.printf("[warning] %s\n", msg);
            }
        } catch (JSONException e) {
        }
    }
}
