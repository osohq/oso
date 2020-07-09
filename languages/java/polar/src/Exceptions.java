import org.json.*;
import java.util.*;

public class Exceptions {
    public static OsoException getJavaError(String polarError) {
        String msg, kind, subkind;
        JSONObject jError, body;
        Map<String, Object> details;

        if (polarError == null) {
            return new Exceptions.FFIErrorNotFound();
        }
        jError = new JSONObject(polarError);
        msg = jError.getString("formatted");
        kind = jError.getJSONObject("kind").keys().next();
        body = jError.getJSONObject("kind").getJSONObject(kind);
        subkind = body.keys().next();
        details = body.getJSONObject(subkind).toMap();

        switch (kind) {
            case "Parse":
                return parseError(kind, msg, details);
            case "Runtime":
                return runtimeError(kind, msg, details);
            case "Operational":
                return operationalError(kind, msg, details);
            case "Parameter":
                return apiError(kind, msg, details);
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

    public static class OsoException extends Exception {
        private static final long serialVersionUID = 1L;
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

    // Expected to find an FFI error to convert into a Java error but found
    // none.
    public static class FFIErrorNotFound extends OsoException {
        private static final long serialVersionUID = 1L;

        public FFIErrorNotFound(String msg, Map<String, Object> details) {
            super(msg, details);
        }

        public FFIErrorNotFound() {
            super(null, null);
        }
    }

    // Generic runtime exception.
    public static class PolarRuntimeException extends OsoException {
        private static final long serialVersionUID = 1L;

        public PolarRuntimeException(String msg, Map<String, Object> details) {
            super(msg, details);
        }

        public PolarRuntimeException(String msg) {
            super(msg);
        }

    }

    // Errors from across the FFI boundary.
    public static class SerializationError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public SerializationError(String msg, Map<String, Object> details) {
            super(msg, details);
        }
    }

    public static class UnsupportedError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public UnsupportedError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class PolarTypeError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public PolarTypeError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class StackOverflowError extends PolarRuntimeException {
        public StackOverflowError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

        private static final long serialVersionUID = 1L;

    }

    // Errors originating from this side of the FFI boundary.
    public static class UnregisteredClassError extends PolarRuntimeException {
        public UnregisteredClassError(String clsName) {
            super(clsName);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class MissingConstructorError extends PolarRuntimeException {
        public MissingConstructorError(String clsName) {
            super("Missing constructor for class " + clsName);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class UnregisteredInstanceError extends PolarRuntimeException {
        public UnregisteredInstanceError(long id) {
            super("Unregistered instance ID: " + id);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class DuplicateInstanceRegistrationError extends PolarRuntimeException {
        public DuplicateInstanceRegistrationError(Long id) {
            super(id.toString());
        }

        private static final long serialVersionUID = 1L;

    }

    public static class InvalidCallError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public InvalidCallError(String msg) {
            super(msg);
        }

    }

    public static class InvalidConstructorError extends PolarRuntimeException {
        public InvalidConstructorError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class InlineQueryFailedError extends PolarRuntimeException {
        public InlineQueryFailedError() {
            super(null, null);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class NullByteInPolarFileError extends PolarRuntimeException {
        public NullByteInPolarFileError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

        private static final long serialVersionUID = 1L;

    }

    public static class UnexpectedPolarTypeError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public UnexpectedPolarTypeError(String type) {
            super(type);
        }

    }

    public static class PolarFileExtensionError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public PolarFileExtensionError() {
            super("Polar files must have a .pol or .polar extension");
        }
    }

    public static class PolarFileNotFoundError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public PolarFileNotFoundError(String filename) {
            super("Could not find file: " + filename);
        }
    }

    public static class DuplicateClassAliasError extends PolarRuntimeException {
        private static final long serialVersionUID = 1L;

        public DuplicateClassAliasError(String alias, String oldClass, String newClass) {
            super("Attempted to alias '" + newClass + "' as '" + alias + "', but " + oldClass
                    + " already has that alias.");
        }

    }

    // # Generic operational exception.
    public static class OperationalError extends OsoException {
        private static final long serialVersionUID = 1L;

        public OperationalError(String msg, Map<String, Object> details) {
            super(msg, details);
        }
    }

    // class UnknownError < OperationalError; end
    public static class UnknownError extends OperationalError {
        private static final long serialVersionUID = 1L;

        public UnknownError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    // Catch-all for a parsing error that doesn't match any of the more specific
    // types.
    public static class ParseError extends OsoException {
        private static final long serialVersionUID = 1L;

        public ParseError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    // class ExtraToken < ParseError; end
    public static class ExtraToken extends ParseError {
        private static final long serialVersionUID = 1L;

        public ExtraToken(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    // class IntegerOverflow < ParseError; end
    public static class IntegerOverflow extends ParseError {
        private static final long serialVersionUID = 1L;

        public IntegerOverflow(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class InvalidTokenCharacter extends ParseError {
        private static final long serialVersionUID = 1L;

        public InvalidTokenCharacter(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class InvalidToken extends ParseError {
        private static final long serialVersionUID = 1L;

        public InvalidToken(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class UnrecognizedEOF extends ParseError {
        private static final long serialVersionUID = 1L;

        public UnrecognizedEOF(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class UnrecognizedToken extends ParseError {
        private static final long serialVersionUID = 1L;

        public UnrecognizedToken(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    // Generic Polar API exception.
    public static class ApiError extends OsoException {
        private static final long serialVersionUID = 1L;

        public ApiError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

    public static class ParameterError extends ApiError {
        private static final long serialVersionUID = 1L;

        public ParameterError(String msg, Map<String, Object> details) {
            super(msg, details);
        }

    }

}
