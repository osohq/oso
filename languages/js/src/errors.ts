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

export class KwargsConstructorError extends PolarError {
  constructor(tag: string) {
    super(
      `To construct a JavaScript instance, use the positional args constructor syntax: new ${tag}(...)`
    );
    Object.setPrototypeOf(this, KwargsConstructorError.prototype);
  }
}

export class InvalidQueryEventError extends PolarError {
  constructor(event: string) {
    super(`Invalid query event: ${event}`);
    Object.setPrototypeOf(this, InvalidQueryEventError.prototype);
  }
}

export class PolarFileAlreadyLoadedError extends PolarError {
  constructor(file: string) {
    super(`File '${file}' already loaded.`);
    Object.setPrototypeOf(this, PolarFileAlreadyLoadedError.prototype);
  }
}

export class PolarFileContentsChangedError extends PolarError {
  constructor(file: string) {
    super(`File '${file}' already loaded, but contents have changed.`);
    Object.setPrototypeOf(this, PolarFileContentsChangedError.prototype);
  }
}

export class PolarFileDuplicateContentError extends PolarError {
  constructor(file: string, existing: string) {
    super(`Content of '${file}' matches the already loaded '${existing}'.`);
    Object.setPrototypeOf(this, PolarFileDuplicateContentError.prototype);
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
