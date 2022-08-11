"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UnexpectedExpressionError = exports.DataFilteringConfigurationError = exports.UnregisteredInstanceError = exports.UnregisteredClassError = exports.PolarFileNotFoundError = exports.PolarFileExtensionError = exports.KwargsError = exports.InvalidQueryEventError = exports.InvalidConstructorError = exports.InvalidIteratorError = exports.InvalidCallError = exports.InvalidAttributeError = exports.InlineQueryFailedError = exports.DuplicateInstanceRegistrationError = exports.DuplicateClassAliasError = exports.PolarError = exports.ForbiddenError = exports.NotFoundError = exports.AuthorizationError = exports.OsoError = void 0;
const helpers_1 = require("./helpers");
/** Base class for all Oso errors. */
class OsoError extends Error {
    constructor(message) {
        // If we ever switch to supporting ES5, we'll have to make sure that
        // OsoError prototypes are properly getting set.
        // See: https://stackoverflow.com/questions/41102060/typescript-extending-error-class/48342359#48342359
        super(message);
    }
}
exports.OsoError = OsoError;
class AuthorizationError extends OsoError {
}
exports.AuthorizationError = AuthorizationError;
class NotFoundError extends AuthorizationError {
    constructor() {
        super('Oso NotFoundError -- The current user does not have permission to read the given resource. You should handle this error by returning a 404 error to the client.');
    }
}
exports.NotFoundError = NotFoundError;
class ForbiddenError extends AuthorizationError {
    constructor() {
        super('Oso ForbiddenError -- The requested action was not allowed for the given resource. You should handle this error by returning a 403 error to the client.');
    }
}
exports.ForbiddenError = ForbiddenError;
/** Base error type. */
class PolarError extends OsoError {
}
exports.PolarError = PolarError;
class DuplicateClassAliasError extends PolarError {
    constructor({ name, cls, existing, }) {
        super(`Attempted to alias ${cls.name} as '${name}', but ${existing.name} already has that alias.`);
    }
}
exports.DuplicateClassAliasError = DuplicateClassAliasError;
class DuplicateInstanceRegistrationError extends PolarError {
    constructor(id) {
        super(`Attempted to register instance ${id}, but an instance with that ID already exists.`);
    }
}
exports.DuplicateInstanceRegistrationError = DuplicateInstanceRegistrationError;
class InlineQueryFailedError extends PolarError {
    constructor(source) {
        super(`Inline query failed: ${source}`);
    }
}
exports.InlineQueryFailedError = InlineQueryFailedError;
class InvalidAttributeError extends PolarError {
    constructor(instance, field) {
        super(`${field} not found on ${helpers_1.repr(instance)}.`);
    }
}
exports.InvalidAttributeError = InvalidAttributeError;
class InvalidCallError extends PolarError {
    constructor(instance, field) {
        super(`${helpers_1.repr(instance)}.${field} is not a function`);
    }
}
exports.InvalidCallError = InvalidCallError;
class InvalidIteratorError extends PolarError {
    constructor(term) {
        super(`${helpers_1.repr(term)} is not iterable`);
    }
}
exports.InvalidIteratorError = InvalidIteratorError;
class InvalidConstructorError extends PolarError {
    constructor(ctor) {
        super(`${helpers_1.repr(ctor)} is not a constructor`);
    }
}
exports.InvalidConstructorError = InvalidConstructorError;
class InvalidQueryEventError extends PolarError {
    constructor(event) {
        super(`Invalid query event: ${event}`);
    }
}
exports.InvalidQueryEventError = InvalidQueryEventError;
class KwargsError extends PolarError {
    constructor() {
        super('JavaScript does not support keyword arguments');
    }
}
exports.KwargsError = KwargsError;
class PolarFileExtensionError extends PolarError {
    constructor(file) {
        super(`Polar files must have .polar extension. Offending file: ${file}`);
    }
}
exports.PolarFileExtensionError = PolarFileExtensionError;
class PolarFileNotFoundError extends PolarError {
    constructor(file) {
        super(`Could not find file: ${file}`);
    }
}
exports.PolarFileNotFoundError = PolarFileNotFoundError;
class UnregisteredClassError extends PolarError {
    constructor(name) {
        super(`Unregistered class: ${name}.`);
    }
}
exports.UnregisteredClassError = UnregisteredClassError;
class UnregisteredInstanceError extends PolarError {
    constructor(id) {
        super(`Unregistered instance: ${id}.`);
    }
}
exports.UnregisteredInstanceError = UnregisteredInstanceError;
class DataFilteringConfigurationError extends PolarError {
    constructor() {
        super("Missing 'adapter' implementation. Did you forget to call `Oso.setDataFilteringAdapter()`?");
    }
}
exports.DataFilteringConfigurationError = DataFilteringConfigurationError;
class UnexpectedExpressionError extends PolarError {
    constructor() {
        super(`Received Expression from Polar VM. The Expression type is only supported when
  using data filtering features. Did you perform an
  operation over an unbound variable in your policy?

  To silence this error and receive an Expression result, pass the
  \`{ acceptExpression: true }\` option to \`Oso.query()\` or \`Oso.queryRule()\`.`);
    }
}
exports.UnexpectedExpressionError = UnexpectedExpressionError;
//# sourceMappingURL=errors.js.map