import { repr } from './helpers';

/** Base error type. */
export class PolarError extends Error {
  constructor(msg: string) {
    super(msg);
    Object.setPrototypeOf(this, PolarError.prototype);
  }
}

export class DuplicateClassAliasError extends PolarError {
  constructor({
    name,
    cls,
    existing,
  }: {
    name: string;
    cls: object;
    existing: object;
  }) {
    super(
      `Attempted to alias ${cls} as '${name}', but ${existing} already has that alias.`
    );
    Object.setPrototypeOf(this, DuplicateClassAliasError.prototype);
  }
}

export class DuplicateInstanceRegistrationError extends PolarError {
  constructor(id: number) {
    super(
      `Attempted to register instance ${id}, but an instance with that ID already exists.`
    );
    Object.setPrototypeOf(this, DuplicateInstanceRegistrationError.prototype);
  }
}

export class InlineQueryFailedError extends PolarError {
  constructor(source: string) {
    super(`Inline query failed: ${source}`);
    Object.setPrototypeOf(this, InlineQueryFailedError.prototype);
  }
}

export class InvalidCallError extends PolarError {
  constructor(instance: any, field: string) {
    super(`${repr(instance)}.${field} is not a function`);
    Object.setPrototypeOf(this, InvalidCallError.prototype);
  }
}

export class InvalidIteratorError extends PolarError {
  constructor(term: any) {
    super(`${term} is not iterable`);
    Object.setPrototypeOf(this, InvalidIteratorError.prototype);
  }
}

export class InvalidConstructorError extends PolarError {
  constructor(ctor: any) {
    super(`${repr(ctor)} is not a constructor`);
    Object.setPrototypeOf(this, InvalidConstructorError.prototype);
  }
}

export class InvalidQueryEventError extends PolarError {
  constructor(event: string) {
    super(`Invalid query event: ${event}`);
    Object.setPrototypeOf(this, InvalidQueryEventError.prototype);
  }
}

export class KwargsError extends PolarError {
  constructor() {
    super('JavaScript does not support keyword arguments');
    Object.setPrototypeOf(this, KwargsError.prototype);
  }
}

export class PolarFileExtensionError extends PolarError {
  constructor(file: string) {
    super(`Polar files must have .polar extension. Offending file: ${file}`);
    Object.setPrototypeOf(this, PolarFileExtensionError.prototype);
  }
}

export class PolarFileNotFoundError extends PolarError {
  constructor(file: string) {
    super(`Could not find file: ${file}`);
    Object.setPrototypeOf(this, PolarFileNotFoundError.prototype);
  }
}

export class UnregisteredClassError extends PolarError {
  constructor(name: string) {
    super(`Unregistered class: ${name}.`);
    Object.setPrototypeOf(this, UnregisteredClassError.prototype);
  }
}

export class UnregisteredInstanceError extends PolarError {
  constructor(id: number) {
    super(`Unregistered instance: ${id}.`);
    Object.setPrototypeOf(this, UnregisteredInstanceError.prototype);
  }
}

export class UnexpectedPolarTypeError extends PolarError {
  constructor() {
    // Doesn't have a tag because it doesn't seem we get this from the wasm API.
    super('Unexpected polar type.');
    Object.setPrototypeOf(this, UnexpectedPolarTypeError.prototype);
  }
}
