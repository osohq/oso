package com.osohq.oso;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;
import jnr.ffi.Struct;
import jnr.ffi.StructLayout;
import jnr.ffi.StructLayout.SignedLong;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

public class Ffi {
  // singleton variable
  private static Ffi ffi = null;

  private PolarLib polarLib;

  // protected static class CResultT extends StructLayout {
  // private T result;
  // private jnr.ffi.Pointer error;

  // // Necessary constructor that takes a Runtime
  // public CResult(jnr.ffi.Runtime runtime) {
  // super(runtime);
  // }

  // public T check() throws Exceptions.OsoException {
  // if (error == null) {
  // return result;
  // } else {
  // java.lang.String s = error.getString(0);
  // // polarLib.string_free(error);
  // throw Exceptions.getJavaError(s);
  // }
  // }
  // }

  public static final class CResultPointer extends Struct {
    public final Pointer result = new Pointer();
    public final Pointer error = new Pointer();

    // Necessary constructor that takes a Runtime
    public CResultPointer(final jnr.ffi.Runtime runtime) {
      super(runtime);
    }

    public jnr.ffi.Pointer check() throws Exceptions.OsoException {
      if (error == null) {
        return result.get();
      } else {
        java.lang.String s = error.get().getString(0);
        // polarLib.string_free(error);
        throw Exceptions.getJavaError(s);
      }
    }
  }

  public static final class CResultInteger extends Struct {
    public final SignedLong result = new SignedLong();
    public final Pointer error = new Pointer();

    // Necessary constructor that takes a Runtime
    public CResultInteger(jnr.ffi.Runtime runtime) {
      super(runtime);
    }

    public long check() throws Exceptions.OsoException {
      if (error == null) {
        return result.get();
      } else {
        java.lang.String s = error.get().getString(0);
        // polarLib.string_free(error);
        throw Exceptions.getJavaError(s);
      }
    }
  }

  protected class Polar {
    private Pointer ptr;

    private Polar(Pointer ptr) {
      this.ptr = ptr;
    }

    protected Pointer get() {
      return ptr;
    }

    protected long newId() {
      return polarLib.polar_get_external_id(ptr);
    }

    protected void load(JSONArray sources) throws Exceptions.OsoException {
      polarLib.polar_load(ptr, sources.toString()).check();
      processMessages();
    }

    protected void clearRules() throws Exceptions.OsoException {
      polarLib.polar_clear_rules(ptr).check();
      processMessages();
    }

    protected Query newQueryFromStr(String queryStr) throws Exceptions.OsoException {
      Pointer queryPtr = polarLib.polar_new_query(ptr, queryStr, 0).check();
      processMessages();
      return new Query(queryPtr);
    }

    protected Query newQueryFromTerm(String queryTerm) throws Exceptions.OsoException {
      Pointer queryPtr = polarLib.polar_new_query_from_term(ptr, queryTerm, 0).check();
      processMessages();
      return new Query(queryPtr);
    }

    protected Query nextInlineQuery() throws Exceptions.OsoException {
      // Don't check result here because the returned Pointer is null to indicate
      // termination
      Pointer p = polarLib.polar_next_inline_query(ptr, 0).check();
      processMessages();
      if (p == null) {
        return null;
      } else {
        return new Query(p);
      }
    }

    protected void registerConstant(String value, String name) throws Exceptions.OsoException {
      CResultInteger result = polarLib.polar_register_constant(ptr, name, value);
      System.out.println(result);
      // if(result == null) {

      // }
      result.check();
    }

    protected void registerMro(String name, String mro) throws Exceptions.OsoException {
      polarLib.polar_register_mro(ptr, name, mro).check();
    }

    protected Pointer nextMessage() throws Exceptions.OsoException {
      return polarLib.polar_next_polar_message(ptr).check();
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

    protected void questionResult(long call_id, int result) throws Exceptions.OsoException {
      polarLib.polar_question_result(ptr, call_id, result).check();
    }

    protected void callResult(long call_id, String value) throws Exceptions.OsoException {
      polarLib.polar_call_result(ptr, call_id, value).check();
    }

    protected void applicationError(String message) throws Exceptions.OsoException {
      polarLib.polar_application_error(ptr, message).check();
    }

    protected QueryEvent nextEvent() throws Exceptions.OsoException {
      Pointer eventPtr = polarLib.polar_next_query_event(ptr).check();
      processMessages();
      return new QueryEvent(eventPtr);
    }

    protected void debugCommand(String value) throws Exceptions.OsoException {
      polarLib.polar_debug_command(ptr, value).check();
      processMessages();
    }

    protected Pointer nextMessage() throws Exceptions.OsoException {
      return polarLib.polar_next_query_message(ptr).check();
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

    protected String source() throws Exceptions.OsoException {
      Pointer sourcePtr = polarLib.polar_query_source_info(ptr).check();
      String source = sourcePtr.getString(0);
      polarLib.string_free(sourcePtr);
      return source;
    }

    protected void bind(String name, String value) throws Exceptions.OsoException {
      polarLib.polar_bind(ptr, name, value).check();
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

  protected static interface PolarLib {
    CResultInteger polar_debug_command(Pointer query_ptr, String value);

    int polar_free(Pointer polar);

    Pointer polar_get_error();

    long polar_get_external_id(Pointer polar_ptr);

    CResultInteger polar_load(Pointer polar_ptr, String sources);

    CResultInteger polar_clear_rules(Pointer polar_ptr);

    Pointer polar_new();

    CResultPointer polar_new_query(Pointer polar_ptr, String query_str, int trace);

    CResultPointer polar_new_query_from_term(Pointer polar_ptr, String query_term, int trace);

    CResultPointer polar_next_inline_query(Pointer polar_ptr, int trace);

    CResultPointer polar_next_query_event(Pointer query_ptr);

    CResultPointer polar_query_from_repl(Pointer polar_ptr);

    CResultInteger polar_question_result(Pointer query_ptr, long call_id, int result);

    CResultInteger polar_call_result(Pointer query_ptr, long call_id, String value);

    CResultInteger polar_application_error(Pointer query_ptr, String message);

    int query_free(Pointer query);

    int string_free(Pointer s);

    CResultInteger polar_register_constant(Pointer polar_ptr, String name, String value);

    CResultInteger polar_register_mro(Pointer polar_ptr, String name, String mro);

    CResultPointer polar_next_polar_message(Pointer polar_ptr);

    CResultPointer polar_next_query_message(Pointer query_ptr);

    CResultPointer polar_query_source_info(Pointer query_ptr);

    CResultInteger polar_bind(Pointer query_ptr, String name, String value);
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

  protected Polar polarNew() {
    return new Polar(polarLib.polar_new());
  }

  protected int stringFree(Pointer s) {
    return polarLib.string_free(s);
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
    } catch (JSONException ignored) {
      throw new Exceptions.OsoException(String.format("Invalid JSON Message: %s", msgStr), null);
    }
  }
}
