/* eslint-disable @typescript-eslint/explicit-module-boundary-types */

export class A {
  a() {
    return 'A';
  }

  x() {
    return this.a();
  }
}

export class BaseActor {
  name: string;

  constructor(name: string) {
    this.name = name;
  }

  widget(): Widget {
    return new Widget('1');
  }

  *widgets(): Generator<Widget> {
    yield new Widget('2');
    yield new Widget('3');
  }
}

export class Animal {
  readonly family: string | undefined;
  readonly genus: string | undefined;
  readonly species: string | undefined;

  constructor({
    family,
    genus,
    species,
  }: {
    family?: string;
    genus?: string;
    species?: string;
  }) {
    this.family = family;
    this.genus = genus;
    this.species = species;
  }
}

export class Async {
  wait(): Promise<number> {
    return new Promise(res => res(1));
  }
}

export class B extends A {
  b() {
    return 'B';
  }

  x() {
    return this.b();
  }
}

export class Bar {
  y() {
    return 'y';
  }
}

export class Belonger {
  groups() {
    return ['engineering', 'social', 'admin'];
  }
}

export class C extends B {
  c() {
    return 'C';
  }

  x() {
    return this.c();
  }
}

export class ConstructorArgs {
  readonly bar: number;
  readonly baz: number;

  constructor(bar: number, baz: number) {
    this.bar = bar;
    this.baz = baz;
  }
}

export class ConstructorNoArgs {}

export class ConstructorMapObjectArgs {
  readonly one?: number;
  readonly two: number;
  readonly three?: number;
  readonly four: number;

  constructor(
    oneMap: Map<'one', number>,
    { two }: { two: number },
    threeMap: Map<'three', number>,
    { four }: { four: number }
  ) {
    this.one = oneMap.get('one');
    this.two = two;
    this.three = threeMap.get('three');
    this.four = four;
  }
}

export class ConstructorAnyArg {
  readonly opts;
  constructor(opts: unknown) {
    this.opts = opts;
  }
}

let counter = 0;

export class Counter {
  static count() {
    return counter;
  }

  constructor() {
    counter += 1;
  }
}

export class Foo {
  readonly a: string;

  constructor(a: string) {
    this.a = a;
  }

  *b() {
    yield 'b';
  }

  c() {
    return 'c';
  }

  d(x: unknown) {
    return x;
  }

  bar() {
    return new Bar();
  }

  e() {
    return [1, 2, 3];
  }

  *f() {
    yield [1, 2, 3];
    yield [4, 5, 6];
    yield 7;
  }

  g() {
    return { hello: 'world' };
  }

  h() {
    return true;
  }
}

export class User {
  readonly name: string;
  special: boolean;

  constructor(name: string) {
    this.name = name;
    this.special = false;
  }
}

export class Widget {
  readonly id: string;

  constructor(id: string) {
    this.id = id;
  }
}

export class X {
  x() {
    return 'X';
  }
}

export class NonIterable {
  constructor() {} // eslint-disable-line @typescript-eslint/no-empty-function
}

export class BarIterator {
  items: number[];
  constructor(items: number[]) {
    this.items = items;
  }
  sum() {
    return this.items.reduce((prev, curr) => prev + curr);
  }
  [Symbol.iterator]() {
    return this.items.values();
  }
}
