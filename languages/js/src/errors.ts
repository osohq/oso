import { repr } from './helpers';
import type { Class } from './types';

/** Base class for all Oso errors. */
export class OsoError extends Error {
  constructor(message?: string) {
    // If we ever switch to supporting ES5, we'll have to make sure that
    // OsoError prototypes are properly getting set.
    // See: https://stackoverflow.com/questions/41102060/typescript-extending-error-class/48342359#48342359
    super(message);
  }
}

export class AuthorizationError extends OsoError {}

export class NotFoundError extends AuthorizationError {
  constructor() {
    super(
      'Oso NotFoundError -- The current user does not have permission to read the given resource. You should handle this error by returning a 404 error to the client.'
    );
  }
}

export class ForbiddenError extends AuthorizationError {
  constructor() {
    super(
      'Oso ForbiddenError -- The requested action was not allowed for the given resource. You should handle this error by returning a 403 error to the client.'
    );
  }
}

/** Base error type. */
export class PolarError extends OsoError {}

export class DuplicateClassAliasError extends PolarError {
  constructor({
    name,
    cls,
    existing,
  }: {
    name: string;
    cls: Class;
    existing: Class;
  }) {
    super(
      `Attempted to alias ${cls.name} as '${name}', but ${existing.name} already has that alias.`
    );
  }
}

export class DuplicateInstanceRegistrationError extends PolarError {
  constructor(id: number) {
    super(
      `Attempted to register instance ${id}, but an instance with that ID already exists.`
    );
  }
}

export class InlineQueryFailedError extends PolarError {
  constructor(source: string) {
    super(`Inline query failed: ${source}`);
  }
}

export class InvalidAttributeError extends PolarError {
  constructor(instance: unknown, field: string) {
    super(`${field} not found on ${repr(instance)}.`);
  }
}

export class InvalidCallError extends PolarError {
  constructor(instance: unknown, field: string) {
    super(`${repr(instance)}.${field} is not a function`);
  }
}

export class InvalidIteratorError extends PolarError {
  constructor(term: unknown) {
    super(`${repr(term)} is not iterable`);
  }
}

export class InvalidConstructorError extends PolarError {
  constructor(ctor: unknown) {
    super(`${repr(ctor)} is not a constructor`);
  }
}

export class InvalidQueryEventError extends PolarError {
  constructor(event: string) {
    super(`Invalid query event: ${event}`);
  }
}

export class KwargsError extends PolarError {
  constructor() {
    super('JavaScript does not support keyword arguments');
  }
}

export class PolarFileExtensionError extends PolarError {
  constructor(file: string) {
    super(`Polar files must have .polar extension. Offending file: ${file}`);
  }
}

export class PolarFileNotFoundError extends PolarError {
  constructor(file: string) {
    super(`Could not find file: ${file}`);
  }
}

export class UnregisteredClassError extends PolarError {
  constructor(name: string) {
    super(`Unregistered class: ${name}.`);
  }
}

export class UnregisteredInstanceError extends PolarError {
  constructor(id: number) {
    super(`Unregistered instance: ${id}.`);
  }
}

export class DataFilteringConfigurationError extends PolarError {
  constructor(fn: 'buildQuery' | 'execQuery' | 'combineQuery') {
    super(
      `Missing '${fn}' implementation. Did you forget to call \`Oso.setDataFilteringQueryDefaults()\`?`
    );
  }
}
