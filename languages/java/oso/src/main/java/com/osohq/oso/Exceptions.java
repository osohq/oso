package com.osohq.oso;

import java.util.*;
import org.json.*;

@SuppressWarnings("serial")
public class Exceptions {
  public static OsoException getJavaError(String polarError) {
    String msg, kind, subkind;
    JSONObject jError, body;
    Map<String, Object> details = null;

    if (polarError == null) {
      return new Exceptions.FFIErrorNotFound();
    }
    jError = new JSONObject(polarError);
    msg = jError.getString("formatted");
    kind = jError.getJSONObject("kind").keys().next();
    try {
      body = jError.getJSONObject("kind").getJSONObject(kind);
      subkind = body.keys().next();
      details = body.getJSONObject(subkind).toMap();
    } catch (JSONException e) {
      subkind = jError.getJSONObject("kind").getString(kind);
    }

    switch (kind) {
      case "Parse":
        return parseError(subkind, msg, details);
      case "Runtime":
        return runtimeError(subkind, msg, details);
      case "Operational":
        return operationalError(subkind, msg, details);
      case "Parameter":
        return apiError(subkind, msg, details);
      default:
        return new OsoException(msg, details);
    }
  }

  private static OsoException parseError(String kind, String msg, Map<String, Object> details) {
    switch (kind) {
      case "ExtraToken":
        return new ExtraToken(msg, details);
      case "IntegerOverflow":
        return new IntegerOverflow(msg, details);
      case "InvalidToken":
        return new InvalidToken(msg, details);
      case "InvalidTokenCharacter":
        return new InvalidTokenCharacter(msg, details);
      case "UnrecognizedEOF":
        return new UnrecognizedEOF(msg, details);
      case "UnrecognizedToken":
        return new UnrecognizedToken(msg, details);
      default:
        return new ParseError(msg, details);
    }
  }

  private static PolarRuntimeException runtimeError(String kind, String msg, Map<String, Object> details) {
    switch (kind) {
      case "Serialization":
        return new SerializationError(msg, details);
      case "Unsupported":
        return new UnsupportedError(msg, details);
      case "TypeError":
        return new PolarTypeError(msg, details);
      case "StackOverflow":
        return new StackOverflowError(msg, details);
      default:
        return new PolarRuntimeException(msg, details);
    }
  }

  private static OperationalError operationalError(String kind, String msg, Map<String, Object> details) {
    switch (kind) {
      case "Unknown":
        return new UnknownError(msg, details);
      default:
        return new OperationalError(msg, details);
    }
  }

  private static ApiError apiError(String kind, String msg, Map<String, Object> details) {
    switch (kind) {
      case "Parameter":
        return new ParameterError(msg, details);
      default:
        return new ApiError(msg, details);
    }
  }

  public static class OsoException extends RuntimeException {
    private Map<String, Object> details;

    public OsoException(String msg, Map<String, Object> details) {
      super(msg);
      this.details = details;
    }

    public OsoException(String msg) {
      super(msg);
    }

    public Map<String, Object> getDetails() {
      return details;
    }
  }

  /**
   * Expected to find an FFI error to convert into a Java error but found none.
   */
  public static class FFIErrorNotFound extends OsoException {
    public FFIErrorNotFound(String msg, Map<String, Object> details) {
      super(msg, details);
    }

    public FFIErrorNotFound() {
      super(null, null);
    }
  }

  /** Generic runtime exception. */
  public static class PolarRuntimeException extends OsoException {
    public PolarRuntimeException(String msg, Map<String, Object> details) {
      super(msg, details);
    }

    public PolarRuntimeException(String msg) {
      super(msg);
    }
  }

  // Errors from across the FFI boundary.

  public static class SerializationError extends PolarRuntimeException {
    public SerializationError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnsupportedError extends PolarRuntimeException {
    public UnsupportedError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class PolarTypeError extends PolarRuntimeException {
    public PolarTypeError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class StackOverflowError extends PolarRuntimeException {
    public StackOverflowError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class FileLoadingError extends PolarRuntimeException {
    public FileLoadingError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  // Errors originating from this side of the FFI boundary.

  public static class UnregisteredClassError extends PolarRuntimeException {
    public UnregisteredClassError(String clsName) {
      super(clsName);
    }
  }

  public static class MissingConstructorError extends PolarRuntimeException {
    public MissingConstructorError(String clsName) {
      super("Missing constructor for class " + clsName);
    }
  }

  public static class UnregisteredInstanceError extends PolarRuntimeException {
    public UnregisteredInstanceError(long id) {
      super("Unregistered instance ID: " + id);
    }
  }

  public static class DuplicateInstanceRegistrationError extends PolarRuntimeException {
    public DuplicateInstanceRegistrationError(Long id) {
      super(id.toString());
    }
  }

  public static class InvalidCallError extends PolarRuntimeException {
    public InvalidCallError(String className, String callName, Class<?>... argTypes) {
      super("Invalid call `" + callName + "` on class " + className + ", with argument types " + "`" + argTypes + "`");
    }

    public InvalidCallError(String msg) {
      super(msg);
    }
  }

  public static class InvalidAttributeError extends PolarRuntimeException {
    public InvalidAttributeError(String className, String attrName) {
      super("Invalid attribute `" + attrName + "` on class " + className);
    }

    public InvalidAttributeError(String msg) {
      super(msg);
    }
  }

  public static class InvalidConstructorError extends PolarRuntimeException {
    public InvalidConstructorError(String msg) {
      super(msg);
    }

    public InvalidConstructorError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class InvalidIteratorError extends PolarRuntimeException {
    public InvalidIteratorError(Object value) {
      super(String.format("value %s of type %s is not iterable", value, value.getClass()));
    }
  }

  public static class InstantiationError extends PolarRuntimeException {
    public InstantiationError(String className) {
      super("constructor on class `" + className + "`");
    }

    public InstantiationError(String className, Exception e) {
      super("constructor on class `" + className + "`: " + e.getMessage());
    }
  }

  public static class InlineQueryFailedError extends PolarRuntimeException {
    public InlineQueryFailedError(String source) {
      super("Inline query failed: " + source);
    }
  }

  public static class NullByteInPolarFileError extends PolarRuntimeException {
    public NullByteInPolarFileError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnexpectedPolarTypeError extends PolarRuntimeException {
    public UnexpectedPolarTypeError(String type) {
      super(type);
    }
  }

  public static class PolarFileExtensionError extends PolarRuntimeException {
    public PolarFileExtensionError(String filename) {
      super("Polar files must have .polar extension. Offending file: " + filename);
    }
  }

  public static class PolarFileNotFoundError extends PolarRuntimeException {
    public PolarFileNotFoundError(String filename) {
      super("Could not find file: " + filename);
    }
  }

  public static class DuplicateClassAliasError extends PolarRuntimeException {
    public DuplicateClassAliasError(String alias, String oldClass, String newClass) {
      super("Attempted to alias '" + newClass + "' as '" + alias + "', but " + oldClass + " already has that alias.");
    }
  }

  public static class OperationalError extends OsoException {
    public OperationalError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnknownError extends OperationalError {
    public UnknownError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class ParseError extends OsoException {
    public ParseError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class ExtraToken extends ParseError {
    public ExtraToken(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class IntegerOverflow extends ParseError {
    public IntegerOverflow(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class InvalidTokenCharacter extends ParseError {
    public InvalidTokenCharacter(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class InvalidToken extends ParseError {
    public InvalidToken(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnrecognizedEOF extends ParseError {
    public UnrecognizedEOF(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnrecognizedToken extends ParseError {
    public UnrecognizedToken(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  /** Generic Polar API exception. */
  public static class ApiError extends OsoException {
    public ApiError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class ParameterError extends ApiError {
    public ParameterError(String msg, Map<String, Object> details) {
      super(msg, details);
    }
  }

  public static class UnimplementedOperation extends PolarRuntimeException {
    public UnimplementedOperation(String operation) {
      super(operation + " are unimplemented in the oso Java library");
    }
  }
}
