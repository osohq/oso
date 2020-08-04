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

export class InvalidConstructorError extends Error {
  constructor({ constructor, cls }: { constructor: any; cls: object }) {
    super(
      `${JSON.stringify(constructor)} is not a valid constructor for ${
        cls.constructor.name
      }.`
    );
    Object.setPrototypeOf(this, InvalidConstructorError.prototype);
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
