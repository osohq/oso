import { truncateSync } from 'fs';

import { Polar } from './Polar';
import { Variable } from './Variable';
import {
  map,
  pred,
  query,
  qvar,
  tempFile,
  tempFileFx,
  tempFileGx,
} from '../test/helpers';
import {
  A,
  Actor,
  Animal,
  Async,
  B,
  Bar,
  Belonger,
  C,
  ConstructorArgs,
  ConstructorNoArgs,
  Counter,
  Foo,
  User,
  Widget,
  X,
} from '../test/classes';
import {
  DuplicateClassAliasError,
  InlineQueryFailedError,
  PolarFileContentsChangedError,
  PolarFileDuplicateContentError,
  PolarFileExtensionError,
  PolarFileAlreadyLoadedError,
  PolarFileNotFoundError,
} from './errors';

test('it works', () => {
  const p = new Polar();
  p.loadStr('f(1);');
  const result = query(p, 'f(x)');
  expect(result).toStrictEqual([map({ x: 1 })]);
});

describe('#registerClass', () => {
  test('can specialize on a registered class', () => {
    const p = new Polar();
    p.registerClass(User);
    p.loadStr('allow(u: User, 1, 2) if u.name = "alice";');
    const result = query(p, pred('allow', new User('alice'), 1, 2));
    expect(result).toStrictEqual([map()]);
  });

  test('errors when registering the same class twice', () => {
    const p = new Polar();
    expect(() => p.registerClass(Actor)).not.toThrow();
    expect(() => p.registerClass(Actor)).toThrow(DuplicateClassAliasError);
  });

  test('errors when registering the same alias twice', () => {
    const p = new Polar();
    expect(() => p.registerClass(Actor)).not.toThrow();
    expect(() => p.registerClass(User, 'Actor')).toThrow(
      DuplicateClassAliasError
    );
  });

  test('can register the same class under different aliases', () => {
    const p = new Polar();
    p.registerClass(ConstructorNoArgs, 'A');
    p.registerClass(ConstructorNoArgs, 'B');
    p.registerClass(ConstructorArgs, 'C');
    expect(() =>
      p.loadStr('?= new A() = new B() and not new A() = new C();')
    ).not.toThrow();
  });

  test('registers a JS class with Polar', () => {
    const p = new Polar();
    p.registerClass(Foo);
    p.registerClass(Bar);
    expect(qvar(p, 'new Foo("A").a = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new Foo("A").a() = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new Foo("A").b = x', 'x', true)).toStrictEqual('b');
    expect(qvar(p, 'new Foo("A").b() = x', 'x', true)).toStrictEqual('b');
    expect(qvar(p, 'new Foo("A").c = x', 'x', true)).toStrictEqual('c');
    expect(qvar(p, 'new Foo("A").c() = x', 'x', true)).toStrictEqual('c');
    expect(qvar(p, 'new Foo("A") = f and f.a() = x', 'x', true)).toStrictEqual(
      'A'
    );
    expect(qvar(p, 'new Foo("A").bar().y() = x', 'x', true)).toStrictEqual('y');
    expect(qvar(p, 'new Foo("A").e = x', 'x')).toStrictEqual([[1, 2, 3]]);
    expect(qvar(p, 'new Foo("A").f = x', 'x')).toStrictEqual([
      [1, 2, 3],
      [4, 5, 6],
      7,
    ]);
    expect(qvar(p, 'new Foo("A").g.hello = x', 'x', true)).toStrictEqual(
      'world'
    );
    expect(qvar(p, 'new Foo("A").h = x', 'x', true)).toBe(true);
  });

  test('respects the JS prototype hierarchy for class specialization', () => {
    const p = new Polar();
    p.registerClass(A);
    p.registerClass(B);
    p.registerClass(C);
    p.registerClass(X);
    p.loadStr(`
      test(_: A);
      test(_: B);

      try(_: B, res) if res = 2;
      try(_: C, res) if res = 3;
      try(_: A, res) if res = 1;
    `);
    expect(qvar(p, 'new A().a = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new A().x = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new B().a = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new B().b = x', 'x', true)).toStrictEqual('B');
    expect(qvar(p, 'new B().x = x', 'x', true)).toStrictEqual('B');
    expect(qvar(p, 'new C().a = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new C().b = x', 'x', true)).toStrictEqual('B');
    expect(qvar(p, 'new C().c = x', 'x', true)).toStrictEqual('C');
    expect(qvar(p, 'new C().x = x', 'x', true)).toStrictEqual('C');
    expect(qvar(p, 'new X().x = x', 'x', true)).toStrictEqual('X');

    expect(query(p, 'test(new A())')).toHaveLength(1);
    expect(query(p, 'test(new B())')).toHaveLength(2);

    expect(qvar(p, 'try(new A(), x)', 'x')).toStrictEqual([1]);
    expect(qvar(p, 'try(new B(), x)', 'x')).toStrictEqual([2, 1]);
    expect(qvar(p, 'try(new C(), x)', 'x')).toStrictEqual([3, 2, 1]);
    expect(qvar(p, 'try(new X(), x)', 'x')).toStrictEqual([]);
  });

  describe('animal tests', () => {
    const wolf =
      'new Animal({species: "canis lupus", genus: "canis", family: "canidae"})';
    const dog =
      'new Animal({species: "canis familiaris", genus: "canis", family: "canidae"})';
    const canine = 'new Animal({genus: "canis", family: "canidae"})';
    const canid = 'new Animal({family: "canidae"})';
    const animal = 'new Animal({})';

    test('can unify instances', () => {
      const p = new Polar();
      p.registerClass(Animal);
      p.loadStr(`
          yup() if new Animal({family: "steve"}) = new Animal({family: "steve"});
          nope() if new Animal({family: "steve"}) = new Animal({family: "gabe"});
        `);
      expect(query(p, 'yup()')).toStrictEqual([map()]);
      expect(query(p, 'nope()')).toStrictEqual([]);
    });

    test('can specialize on dict fields', () => {
      const p = new Polar();
      p.registerClass(Animal);
      p.loadStr(`
          what_is(_: {genus: "canis"}, r) if r = "canine";
          what_is(_: {species: "canis lupus", genus: "canis"}, r) if r = "wolf";
          what_is(_: {species: "canis familiaris", genus: "canis"}, r) if r = "dog";
        `);
      expect(qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf',
        'canine',
      ]);
      expect(qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog',
        'canine',
      ]);
      expect(qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual(['canine']);
    });

    test('can specialize on class fields', () => {
      const p = new Polar();
      p.registerClass(Animal);
      p.loadStr(`
          what_is(_: Animal, r) if r = "animal";
          what_is(_: Animal{genus: "canis"}, r) if r = "canine";
          what_is(_: Animal{family: "canidae"}, r) if r = "canid";
          what_is(_: Animal{species: "canis lupus", genus: "canis"}, r) if r = "wolf";
          what_is(_: Animal{species: "canis familiaris", genus: "canis"}, r) if r = "dog";
          what_is(_: Animal{species: s, genus: "canis"}, r) if r = s;
        `);
      expect(qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf',
        'canis lupus',
        'canine',
        'canid',
        'animal',
      ]);
      expect(qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog',
        'canis familiaris',
        'canine',
        'canid',
        'animal',
      ]);
      expect(qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual([
        'canine',
        'canid',
        'animal',
      ]);
      expect(qvar(p, `what_is(${canid}, r)`, 'r')).toStrictEqual([
        'canid',
        'animal',
      ]);
      expect(qvar(p, `what_is(${animal}, r)`, 'r')).toStrictEqual(['animal']);
    });

    test('can specialize with a mix of class and dict fields', () => {
      const p = new Polar();
      p.registerClass(Animal);
      p.loadStr(`
          what_is(_: Animal, r) if r = "animal_class";
          what_is(_: Animal{genus: "canis"}, r) if r = "canine_class";
          what_is(_: {genus: "canis"}, r) if r = "canine_dict";
          what_is(_: Animal{family: "canidae"}, r) if r = "canid_class";
          what_is(_: {species: "canis lupus", genus: "canis"}, r) if r = "wolf_dict";
          what_is(_: {species: "canis familiaris", genus: "canis"}, r) if r = "dog_dict";
          what_is(_: Animal{species: "canis lupus", genus: "canis"}, r) if r = "wolf_class";
          what_is(_: Animal{species: "canis familiaris", genus: "canis"}, r) if r = "dog_class";
        `);

      const wolf_dict =
        '{species: "canis lupus", genus: "canis", family: "canidae"}';
      const dog_dict =
        '{species: "canis familiaris", genus: "canis", family: "canidae"}';
      const canine_dict = '{genus: "canis", family: "canidae"}';

      // test rule ordering for instances
      expect(qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf_class',
        'canine_class',
        'canid_class',
        'animal_class',
        'wolf_dict',
        'canine_dict',
      ]);
      expect(qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog_class',
        'canine_class',
        'canid_class',
        'animal_class',
        'dog_dict',
        'canine_dict',
      ]);
      expect(qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual([
        'canine_class',
        'canid_class',
        'animal_class',
        'canine_dict',
      ]);

      // test rule ordering for dicts
      expect(qvar(p, `what_is(${wolf_dict}, r)`, 'r')).toStrictEqual([
        'wolf_dict',
        'canine_dict',
      ]);
      expect(qvar(p, `what_is(${dog_dict}, r)`, 'r')).toStrictEqual([
        'dog_dict',
        'canine_dict',
      ]);
      expect(qvar(p, `what_is(${canine_dict}, r)`, 'r')).toStrictEqual([
        'canine_dict',
      ]);
    });
  });
});

describe('conversions between JS + Polar values', () => {
  test('returns JS instances from external calls', () => {
    const actor = new Actor('sam');
    const widget = new Widget('1');
    const p = new Polar();
    p.loadStr('allow(actor, resource) if actor.widget.id = resource.id;');
    const result = Array.from(p.queryRule('allow', actor, widget));
    expect(result).toStrictEqual([map()]);
  });

  test('handles Generator external call results', () => {
    const actor = new Actor('sam');
    const p = new Polar();
    p.loadStr('widgets(actor, x) if x = actor.widgets.id;');
    const result = Array.from(p.queryRule('widgets', actor, new Variable('x')));
    expect(result).toStrictEqual([map({ x: '2' }), map({ x: '3' })]);
  });

  test('caches instances and does not leak them', () => {
    const p = new Polar();
    p.registerClass(Counter);
    p.loadStr('f(c: Counter) if Counter.count > 0;');
    expect(Counter.count()).toBe(0);
    const c = new Counter();
    expect(Counter.count()).toBe(1);
    expect(Array.from(p.queryRule('f', c))).toStrictEqual([map()]);
    expect(Counter.count()).toBe(1);
    // There are 7 classes registered in the Polar instance cache, including
    // the Counter class.
    expect(p.__host().hasInstance(7)).toBe(true);
    // The Counter instance is cached in the Query instance cache, which only
    // lives as long as the query. It's not cached in the Polar instance cache.
    expect(p.__host().hasInstance(8)).toBe(false);
  });
});

describe('#loadFile', () => {
  test('loads a Polar file', () => {
    const p = new Polar();
    p.loadFile(tempFileFx());
    expect(qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
  });

  test('passes the filename across the FFI boundary', () => {
    const p = new Polar();
    const file = tempFile(';', 'invalid.polar');
    expect(() => p.loadFile(file)).toThrow(
      `did not expect to find the token ';' at line 1, column 1 in file ${file}`
    );
  });

  test('throws if given a non-Polar file', () => {
    const p = new Polar();
    expect(() => p.loadFile('other.ext')).toThrow(PolarFileExtensionError);
  });

  test('throws if given a non-existent file', () => {
    const p = new Polar();
    expect(() => p.loadFile('other.polar')).toThrow(PolarFileNotFoundError);
  });

  test('throws if two files with the same contents are loaded', () => {
    const p = new Polar();
    expect(() => p.loadFile(tempFile('', 'a.polar'))).not.toThrow(
      PolarFileDuplicateContentError
    );
    expect(() => p.loadFile(tempFile('', 'b.polar'))).toThrow(
      PolarFileDuplicateContentError
    );
  });

  test('throws if two files with the same name are loaded', () => {
    const p = new Polar();
    const file = tempFile('f();', 'a.polar');
    expect(() => p.loadFile(file)).not.toThrow(PolarFileContentsChangedError);
    truncateSync(file);
    expect(() => p.loadFile(file)).toThrow(PolarFileContentsChangedError);
  });

  test('throws if the same file is loaded twice', () => {
    const p = new Polar();
    const file = tempFileFx();
    expect(() => p.loadFile(file)).not.toThrow(PolarFileAlreadyLoadedError);
    expect(() => p.loadFile(file)).toThrow(PolarFileAlreadyLoadedError);
  });

  test('can load multiple files', () => {
    const p = new Polar();
    p.loadFile(tempFileFx());
    p.loadFile(tempFileGx());
    expect(qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    expect(qvar(p, 'g(x)', 'x')).toStrictEqual([1, 2, 3]);
  });
});

describe('#clear', () => {
  test('clears the KB', () => {
    const p = new Polar();
    p.loadFile(tempFileFx());
    expect(qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    p.clear();
    expect(query(p, 'f(x)')).toStrictEqual([]);
  });
});

describe('#query', () => {
  test('makes basic queries', () => {
    const p = new Polar();
    p.loadStr('f(1);');
    expect(Array.from(p.query('f(1)'))).toStrictEqual([map()]);
  });
});

describe('#queryRule', () => {
  test('makes basic queries', () => {
    const p = new Polar();
    p.loadStr('allow(1, 2, 3);');
    expect(Array.from(p.queryRule('allow', 1, 2, 3))).toStrictEqual([map()]);
  });

  describe('querying for a predicate', () => {
    test('can return a list', () => {
      const p = new Polar();
      p.registerClass(Belonger, 'Actor');
      p.loadStr(
        'allow(actor: Actor, "join", "party") if "social" in actor.groups;'
      );
      expect(
        Array.from(p.queryRule('allow', new Belonger(), 'join', 'party'))
      ).toStrictEqual([map()]);
    });

    test('can handle variables as arguments', () => {
      const p = new Polar();
      p.loadFile(tempFileFx());
      expect(Array.from(p.queryRule('f', new Variable('a')))).toStrictEqual([
        map({ a: 1 }),
        map({ a: 2 }),
        map({ a: 3 }),
      ]);
    });
  });
});

describe('#makeInstance', () => {
  test('handles no args', () => {
    const p = new Polar();
    p.registerClass(ConstructorNoArgs);
    const id = p.__host().makeInstance(ConstructorNoArgs.name, [], 1);
    const instance = p.__host().getInstance(id);
    expect(instance).toStrictEqual(new ConstructorNoArgs());
  });

  test('handles positional args', () => {
    const p = new Polar();
    p.registerClass(ConstructorArgs);
    const one = p.__host().toPolar(1);
    const two = p.__host().toPolar(2);
    const id = p.__host().makeInstance(ConstructorArgs.name, [one, two], 1);
    const instance = p.__host().getInstance(id);
    expect(instance).toStrictEqual(new ConstructorArgs(1, 2));
  });
});

describe('#registerConstant', () => {
  test('works', () => {
    const p = new Polar();
    const d = { a: 1 };
    p.registerConstant('d', d);
    expect(qvar(p, 'd.a = x', 'x')).toStrictEqual([1]);
  });

  describe('can call host language methods', () => {
    test('on strings', () => {
      const p = new Polar();
      expect(query(p, 'x = "abc" and x.indexOf("bc") = 1')).toStrictEqual([
        map({ x: 'abc' }),
      ]);
    });

    test('on numbers', () => {
      const p = new Polar();
      expect(
        query(p, 'f = 314.159 and f.toExponential = "3.14159e+2"')
      ).toStrictEqual([map({ f: 314.159 })]);
    });

    test('on lists', () => {
      const p = new Polar();
      expect(
        query(
          p,
          'l = [1, 2, 3] and l.indexOf(3) = 2 and l.concat([4]) = [1, 2, 3, 4]'
        )
      ).toStrictEqual([map({ l: [1, 2, 3] })]);
    });

    test('on dicts', () => {
      const p = new Polar();
      expect(
        query(p, 'd = {a: 1} and d.a = 1 and d.hasOwnProperty("a")')
      ).toStrictEqual([map({ d: { a: 1 } })]);
    });

    describe('that return undefined', () => {
      xtest('without things blowing up', () => {
        const p = new Polar();
        p.registerConstant('u', {});
        expect(query(p, 'u.x == u.y')).toStrictEqual([map()]);
        expect(() => query(p, 'u.x.y')).toThrow();
      });
    });
  });

  // TODO(gj): Is this expected?
  test('errors when calling host language methods on booleans', () => {
    const p = new Polar();
    expect(() => query(p, 'b = true and b.constructor = Boolean')).toThrow(
      'Type error: can only perform lookups on dicts and instances, this is Boolean(true) at line 1, column 5'
    );
  });

  test('registering the same constant twice overwrites', () => {
    const p = new Polar();
    p.registerConstant('x', 1);
    p.registerConstant('x', 2);
    expect(() => p.loadStr('?= x == 2;')).not.toThrow();
  });
});

describe('unifying promises', () => {
  test('fails if both promises are the same object', () => {
    const p = new Polar();
    p.registerClass(Async);
    const result = Array.from(p.query('new Async().wait() = x and x = x'));
    expect(result).toStrictEqual([]);
  });

  test('fails if the promises are different objects', () => {
    const p = new Polar();
    p.registerClass(Async);
    const result = Array.from(
      p.query('new Async().wait() = new Async().wait()')
    );
    expect(result).toStrictEqual([]);
  });
});

describe('errors', () => {
  describe('with inline queries', () => {
    test('succeeds if all inline queries succeed', () => {
      const p = new Polar();
      expect(() =>
        p.loadStr('f(1); f(2); ?= f(1); ?= not f(3);')
      ).not.toThrow();
    });

    test('fails if an inline query fails', () => {
      const p = new Polar();
      expect(() => p.loadStr('g(1); ?= g(2);')).toThrow(InlineQueryFailedError);
    });
  });

  describe('when parsing', () => {
    test('raises on IntegerOverflow errors', () => {
      const p = new Polar();
      const int = '18446744073709551616';
      const rule = `f(a) if a = ${int};`;
      expect(() => p.loadStr(rule)).toThrow(
        `'${int}' caused an integer overflow at line 1, column 13`
      );
    });

    test('raises on InvalidTokenCharacter errors', () => {
      const p = new Polar();
      const rule = `
        f(a) if a = "this is not
        allowed";
      `;
      expect(() => p.loadStr(rule)).toThrow(
        "'\\n' is not a valid character. Found in this is not at line 2, column 33"
      );
    });

    test.todo('raises on InvalidToken');

    test('raises on UnrecognizedEOF errors', () => {
      const p = new Polar();
      const rule = 'f(a)';
      expect(() => p.loadStr(rule)).toThrow(
        'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 5'
      );
    });

    test('raises on UnrecognizedToken errors', () => {
      const p = new Polar();
      const rule = '1;';
      expect(() => p.loadStr(rule)).toThrow(
        "did not expect to find the token '1' at line 1, column 1"
      );
    });

    test.todo('raises on ExtraToken');
  });

  describe('runtime errors', () => {
    test('include a stack trace', () => {
      const p = new Polar();
      p.loadStr('foo(a,b) if a in b;');
      expect(() => query(p, 'foo(1,2)')).toThrow(
        `trace (most recent evaluation last):
  in query at line 1, column 1
    foo(1, 2)
  in rule foo at line 1, column 13
    _a_3 in _b_4
  in rule foo at line 1, column 13
    _a_3 in _b_4
Type error: can only use \`in\` on a list, this is Variable(Symbol("_a_3")) at line 1, column 13`
      );
    });

    test('work for lookups', () => {
      const p = new Polar();
      p.registerClass(A);
      expect(query(p, 'new A() = {bar: "bar"}')).toStrictEqual([]);
      expect(() => query(p, 'new A().bar = "bar"')).toThrow(
        "Application error: Property 'bar' does not exist on {} at line 1, column 1"
      );
    });
  });
});

describe('unbound variables', () => {
  test('returns unbound properly', () => {
    const p = new Polar();
    p.loadStr('rule(x, y) if y = 1;');

    const result = query(p, 'rule(x, y)')[0];

    expect(result.get('y')).toBe(1);
    expect(result.get('x')).toBeInstanceOf(Variable);
  });
});
