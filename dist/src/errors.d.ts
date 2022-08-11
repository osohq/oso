import type { Class } from './types';
/** Base class for all Oso errors. */
export declare class OsoError extends Error {
    constructor(message?: string);
}
export declare class AuthorizationError extends OsoError {
}
export declare class NotFoundError extends AuthorizationError {
    constructor();
}
export declare class ForbiddenError extends AuthorizationError {
    constructor();
}
/** Base error type. */
export declare class PolarError extends OsoError {
}
export declare class DuplicateClassAliasError extends PolarError {
    constructor({ name, cls, existing, }: {
        name: string;
        cls: Class;
        existing: Class;
    });
}
export declare class DuplicateInstanceRegistrationError extends PolarError {
    constructor(id: number);
}
export declare class InlineQueryFailedError extends PolarError {
    constructor(source: string);
}
export declare class InvalidAttributeError extends PolarError {
    constructor(instance: unknown, field: string);
}
export declare class InvalidCallError extends PolarError {
    constructor(instance: unknown, field: string);
}
export declare class InvalidIteratorError extends PolarError {
    constructor(term: unknown);
}
export declare class InvalidConstructorError extends PolarError {
    constructor(ctor: unknown);
}
export declare class InvalidQueryEventError extends PolarError {
    constructor(event: string);
}
export declare class KwargsError extends PolarError {
    constructor();
}
export declare class PolarFileExtensionError extends PolarError {
    constructor(file: string);
}
export declare class PolarFileNotFoundError extends PolarError {
    constructor(file: string);
}
export declare class UnregisteredClassError extends PolarError {
    constructor(name: string);
}
export declare class UnregisteredInstanceError extends PolarError {
    constructor(id: number);
}
export declare class DataFilteringConfigurationError extends PolarError {
    constructor();
}
export declare class UnexpectedExpressionError extends PolarError {
    constructor();
}
