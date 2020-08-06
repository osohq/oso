export class DuplicateClassAliasError extends Error {
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

export class DuplicateInstanceRegistrationError extends Error {
  constructor(id: bigint) {
    super(
      `Attempted to register instance ${id}, but an instance with that ID already exists.`
    );
    Object.setPrototypeOf(this, DuplicateInstanceRegistrationError.prototype);
  }
}

export class InvalidCallError extends Error {
  constructor(attr: string, instance: any) {
    super(`Property '${attr}' does not exist on ${JSON.stringify(instance)}`);
    Object.setPrototypeOf(this, InvalidCallError.prototype);
  }
}

export class InvalidConstructorError extends Error {
  constructor({ constructor, cls }: { constructor: any; cls: object }) {
    let stringified;
    if (typeof constructor === 'function') {
      stringified = constructor.toString();
    } else {
      stringified = JSON.stringify(constructor);
    }
    super(
      `${stringified} is not a valid constructor for ${cls.constructor.name}.`
    );
    Object.setPrototypeOf(this, InvalidConstructorError.prototype);
  }
}

export class InvalidQueryEventError extends Error {
  constructor(event: string) {
    super(`Invalid query event: ${event}`);
    Object.setPrototypeOf(this, InvalidQueryEventError.prototype);
  }
}

export class MissingConstructorError extends Error {
  constructor(name: string) {
    super(`Missing constructor for class: ${name}.`);
    Object.setPrototypeOf(this, MissingConstructorError.prototype);
  }
}

export class UnregisteredClassError extends Error {
  constructor(name: string) {
    super(`Unregistered class: ${name}.`);
    Object.setPrototypeOf(this, UnregisteredClassError.prototype);
  }
}

export class UnregisteredInstanceError extends Error {
  constructor(id: bigint) {
    super(`Unregistered instance: ${id}.`);
    Object.setPrototypeOf(this, UnregisteredInstanceError.prototype);
  }
}
