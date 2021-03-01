import { Polar } from './Polar';
import { Variable } from './Variable';
import {
  map,
  pred,
  query,
  queryRule,
  qvar,
  tempFile,
  tempFileFx,
  tempFileGx,
  truncate,
} from '../test/helpers';
import {
  A,
  Actor,
  Animal,
  B,
  Bar,
  BarIterator,
  Belonger,
  C,
  ConstructorArgs,
  ConstructorNoArgs,
  Counter,
  Foo,
  NonIterable,
  User,
  Widget,
  X,
} from '../test/classes';
import {
  DuplicateClassAliasError,
  InlineQueryFailedError,
  InvalidConstructorError,
  KwargsError,
  PolarFileNotFoundError,
  PolarFileExtensionError,
  InvalidIteratorError,
  UnexpectedPolarTypeError,
} from './errors';

test('it works', async () => {
  const p = new Polar();
  await p.loadStr('f(1);');
  const result = await query(p, 'f(x)');
  expect(result).toStrictEqual([map({ x: 1 })]);
});

describe('#registerClass', () => {
  test('can specialize on a registered class', async () => {
    const p = new Polar();
    p.registerClass(User);
    await p.loadStr('allow(u: User, 1, 2) if u.name = "alice";');
    const result = await query(p, pred('allow', new User('alice'), 1, 2));
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

  test('can register the same class under different aliases', async () => {
    const p = new Polar();
    p.registerClass(A, 'A');
    p.registerClass(A, 'B');
    expect(await query(p, 'new A().a() = new B().a()')).toStrictEqual([map()]);
  });

  test('registers a JS class with Polar', async () => {
    const p = new Polar();
    p.registerClass(Foo);
    p.registerClass(Bar);
    expect(await qvar(p, 'new Foo("A").a = x', 'x', true)).toStrictEqual('A');
    expect(qvar(p, 'new Foo("A").a() = x', 'x', true)).rejects.toThrow(
      `trace (most recent evaluation last):
  in query at line 1, column 1
    new Foo(\"A\").a() = x
  in query at line 1, column 1
    new Foo(\"A\").a() = x
  in query at line 1, column 1
    new Foo(\"A\").a()
Application error: Foo { a: 'A' }.a is not a function at line 1, column 1`
    );
    expect(qvar(p, 'x in new Foo("A").b', 'x', true)).rejects.toThrow(
      'function is not iterable'
    );
    expect(await qvar(p, 'x in new Foo("A").b()', 'x', true)).toStrictEqual(
      'b'
    );
    expect(await qvar(p, 'new Foo("A").c = x', 'x', true)).not.toStrictEqual(
      'c'
    );
    expect(await qvar(p, 'new Foo("A").c() = x', 'x', true)).toStrictEqual('c');
    expect(
      await qvar(p, 'new Foo("A") = f and f.a = x', 'x', true)
    ).toStrictEqual('A');
    expect(
      await qvar(p, 'new Foo("A").bar().y() = x', 'x', true)
    ).toStrictEqual('y');
    expect(await qvar(p, 'new Foo("A").e() = x', 'x')).toStrictEqual([
      [1, 2, 3],
    ]);
    expect(await qvar(p, 'x in new Foo("A").f()', 'x')).toStrictEqual([
      [1, 2, 3],
      [4, 5, 6],
      7,
    ]);
    expect(
      await qvar(p, 'new Foo("A").g().hello = x', 'x', true)
    ).toStrictEqual('world');
    expect(await qvar(p, 'new Foo("A").h() = x', 'x', true)).toBe(true);
  });

  test('respects the JS prototype hierarchy for class specialization', async () => {
    const p = new Polar();
    p.registerClass(A);
    p.registerClass(B);
    p.registerClass(C);
    p.registerClass(X);
    await p.loadStr(`
      test(_: A);
      test(_: B);

      try(_: B, res) if res = 2;
      try(_: C, res) if res = 3;
      try(_: A, res) if res = 1;
    `);
    expect(await qvar(p, 'new A().a() = x', 'x', true)).toStrictEqual('A');
    expect(await qvar(p, 'new A().x() = x', 'x', true)).toStrictEqual('A');
    expect(await qvar(p, 'new B().a() = x', 'x', true)).toStrictEqual('A');
    expect(await qvar(p, 'new B().b() = x', 'x', true)).toStrictEqual('B');
    expect(await qvar(p, 'new B().x() = x', 'x', true)).toStrictEqual('B');
    expect(await qvar(p, 'new C().a() = x', 'x', true)).toStrictEqual('A');
    expect(await qvar(p, 'new C().b() = x', 'x', true)).toStrictEqual('B');
    expect(await qvar(p, 'new C().c() = x', 'x', true)).toStrictEqual('C');
    expect(await qvar(p, 'new C().x() = x', 'x', true)).toStrictEqual('C');
    expect(await qvar(p, 'new X().x() = x', 'x', true)).toStrictEqual('X');

    expect(await query(p, 'test(new A())')).toHaveLength(1);
    expect(await query(p, 'test(new B())')).toHaveLength(2);

    expect(await qvar(p, 'try(new A(), x)', 'x')).toStrictEqual([1]);
    expect(await qvar(p, 'try(new B(), x)', 'x')).toStrictEqual([2, 1]);
    expect(await qvar(p, 'try(new C(), x)', 'x')).toStrictEqual([3, 2, 1]);
    expect(await qvar(p, 'try(new X(), x)', 'x')).toStrictEqual([]);
  });

  test('rejects keyword arguments in method calls', () => {
    const p = new Polar();
    p.registerClass(A);
    expect(query(p, 'x = (new A()).a(arg: 1)')).rejects.toThrow(KwargsError);
  });

  describe('animal tests', () => {
    const wolf =
      'new Animal({species: "canis lupus", genus: "canis", family: "canidae"})';
    const dog =
      'new Animal({species: "canis familiaris", genus: "canis", family: "canidae"})';
    const canine = 'new Animal({genus: "canis", family: "canidae"})';
    const canid = 'new Animal({family: "canidae"})';
    const animal = 'new Animal({})';

    test('can unify instances with a custom equality function', async () => {
      const p = new Polar({ equalityFn: (x, y) => x.family === y.family });
      p.registerClass(Animal);
      await p.loadStr(`
          yup() if new Animal({family: "steve"}) = new Animal({family: "steve"});
          nope() if new Animal({family: "steve"}) = new Animal({family: "gabe"});
        `);
      expect(await query(p, 'yup()')).toStrictEqual([map()]);
      expect(await query(p, 'nope()')).toStrictEqual([]);
    });

    test('can specialize on dict fields', async () => {
      const p = new Polar();
      p.registerClass(Animal);
      await p.loadStr(`
          what_is(_: {genus: "canis"}, r) if r = "canine";
          what_is(_: {species: "canis lupus", genus: "canis"}, r) if r = "wolf";
          what_is(_: {species: "canis familiaris", genus: "canis"}, r) if r = "dog";
        `);
      expect(await qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf',
        'canine',
      ]);
      expect(await qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog',
        'canine',
      ]);
      expect(await qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual([
        'canine',
      ]);
    });

    test('can specialize on class fields', async () => {
      const p = new Polar();
      p.registerClass(Animal);
      await p.loadStr(`
          what_is(_: Animal, r) if r = "animal";
          what_is(_: Animal{genus: "canis"}, r) if r = "canine";
          what_is(_: Animal{family: "canidae"}, r) if r = "canid";
          what_is(_: Animal{species: "canis lupus", genus: "canis"}, r) if r = "wolf";
          what_is(_: Animal{species: "canis familiaris", genus: "canis"}, r) if r = "dog";
          what_is(_: Animal{species: s, genus: "canis"}, r) if r = s;
        `);
      expect(await qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf',
        'canis lupus',
        'canine',
        'canid',
        'animal',
      ]);
      expect(await qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog',
        'canis familiaris',
        'canine',
        'canid',
        'animal',
      ]);
      expect(await qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual([
        undefined, // Canine has no species, so looking up the 'species' prop returns undefined.
        'canine',
        'canid',
        'animal',
      ]);
      expect(await qvar(p, `what_is(${canid}, r)`, 'r')).toStrictEqual([
        'canid',
        'animal',
      ]);
      expect(await qvar(p, `what_is(${animal}, r)`, 'r')).toStrictEqual([
        'animal',
      ]);
    });

    test('can specialize with a mix of class and dict fields', async () => {
      const p = new Polar();
      p.registerClass(Animal);
      await p.loadStr(`
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
      expect(await qvar(p, `what_is(${wolf}, r)`, 'r')).toStrictEqual([
        'wolf_class',
        'canine_class',
        'canid_class',
        'animal_class',
        'wolf_dict',
        'canine_dict',
      ]);
      expect(await qvar(p, `what_is(${dog}, r)`, 'r')).toStrictEqual([
        'dog_class',
        'canine_class',
        'canid_class',
        'animal_class',
        'dog_dict',
        'canine_dict',
      ]);
      expect(await qvar(p, `what_is(${canine}, r)`, 'r')).toStrictEqual([
        'canine_class',
        'canid_class',
        'animal_class',
        'canine_dict',
      ]);

      // test rule ordering for dicts
      expect(await qvar(p, `what_is(${wolf_dict}, r)`, 'r')).toStrictEqual([
        'wolf_dict',
        'canine_dict',
      ]);
      expect(await qvar(p, `what_is(${dog_dict}, r)`, 'r')).toStrictEqual([
        'dog_dict',
        'canine_dict',
      ]);
      expect(await qvar(p, `what_is(${canine_dict}, r)`, 'r')).toStrictEqual([
        'canine_dict',
      ]);
    });
  });

  test('errors when passed a non-constructable type', () => {
    expect(() => {
      const p = new Polar();
      // @ts-ignore
      p.registerClass(Math);
    }).toThrow(InvalidConstructorError);
  });
});

describe('conversions between JS + Polar values', () => {
  test('returns JS instances from external calls', async () => {
    const actor = new Actor('sam');
    const widget = new Widget('1');
    const p = new Polar();
    await p.loadStr(
      'allow(actor, resource) if actor.widget().id = resource.id;'
    );
    const result = await queryRule(p, 'allow', actor, widget);
    expect(result).toStrictEqual([map()]);
  });

  test('handles Generator external call results', async () => {
    const actor = new Actor('sam');
    const p = new Polar();
    await p.loadStr('widgets(actor, x) if w in actor.widgets() and x = w.id;');
    const result = await queryRule(p, 'widgets', actor, new Variable('x'));
    expect(result).toStrictEqual([map({ x: '2' }), map({ x: '3' })]);
  });

  describe('caches instances and does not leak them', () => {
    test("instances created in a query don't outlive the query", async () => {
      const p = new Polar();
      p.registerClass(Counter);

      const preLoadInstanceCount = p.__host().instances().length;
      await p.loadStr('f(_: Counter) if Counter.count() > 0;');
      const preQueryInstanceCount = p.__host().instances().length;
      expect(preLoadInstanceCount).toStrictEqual(preQueryInstanceCount);

      expect(Counter.count()).toBe(0);
      const c = new Counter();
      expect(Counter.count()).toBe(1);

      expect(await queryRule(p, 'f', c)).toStrictEqual([map()]);
      const postQueryInstanceCount = p.__host().instances().length;
      expect(preQueryInstanceCount).toStrictEqual(postQueryInstanceCount);

      expect(Counter.count()).toBe(1);
    });
  });
});

describe('#loadFile', () => {
  test('loads a Polar file', async () => {
    const p = new Polar();
    await p.loadFile(await tempFileFx());
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
  });

  test('passes the filename across the FFI boundary', async () => {
    const p = new Polar();
    const file = await tempFile(';', 'invalid.polar');
    expect(p.loadFile(file)).rejects.toThrow(
      `did not expect to find the token ';' at line 1, column 1 in file ${file}`
    );
  });

  test('throws if given a non-Polar file', () => {
    const p = new Polar();
    expect(p.loadFile('other.ext')).rejects.toThrow(PolarFileExtensionError);
  });

  test('throws if given a non-existent file', () => {
    const p = new Polar();
    expect(p.loadFile('other.polar')).rejects.toThrow(PolarFileNotFoundError);
  });

  test('throws if two files with the same contents are loaded', async () => {
    const p = new Polar();
    await expect(
      p.loadFile(await tempFile('', 'a.polar'))
    ).resolves.not.toThrow();
    expect(p.loadFile(await tempFile('', 'b.polar'))).rejects.toThrow(
      /Problem loading file: A file with the same contents as .*b.polar named .*a.polar has already been loaded./
    );
  });

  test('throws if two files with the same name are loaded', async () => {
    const p = new Polar();
    const file = await tempFile('f();', 'a.polar');
    await expect(p.loadFile(file)).resolves.not.toThrow();
    await truncate(file);
    await expect(p.loadFile(file)).rejects.toThrow(
      /Problem loading file: A file with the name .*a.polar, but different contents has already been loaded./
    );
  });

  test('throws if the same file is loaded twice', async () => {
    const p = new Polar();
    const file = await tempFileFx();
    await expect(p.loadFile(file)).resolves.not.toThrow();
    await expect(p.loadFile(file)).rejects.toThrow(
      /Problem loading file: File .*f.polar has already been loaded./
    );
  });

  test('can load multiple files', async () => {
    const p = new Polar();
    await p.loadFile(await tempFileFx());
    await p.loadFile(await tempFileGx());
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    expect(await qvar(p, 'g(x)', 'x')).toStrictEqual([1, 2, 3]);
  });
});

describe('#clearRules', () => {
  test('clears the KB', async () => {
    const p = new Polar();
    await p.loadFile(await tempFileFx());
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    p.clearRules();
    expect(await query(p, 'f(x)')).toStrictEqual([]);
  });

  test('does not clear registered classes', async () => {
    const p = new Polar();
    p.registerClass(Belonger, 'Actor');
    p.clearRules();
    expect(await query(p, 'x = new Actor()')).toHaveLength(1);
  });
});

describe('#query', () => {
  test('makes basic queries', async () => {
    const p = new Polar();
    await p.loadStr('f(1);');
    expect(await query(p, 'f(1)')).toStrictEqual([map()]);
  });
});

describe('#queryRule', () => {
  test('makes basic queries', async () => {
    const p = new Polar();
    await p.loadStr('allow(1, 2, 3);');
    expect(await queryRule(p, 'allow', 1, 2, 3)).toStrictEqual([map()]);
  });

  describe('querying for a predicate', () => {
    test('can return a list', async () => {
      const p = new Polar();
      p.registerClass(Belonger, 'Actor');
      await p.loadStr(
        'allow(actor: Actor, "join", "party") if "social" in actor.groups();'
      );
      expect(
        await queryRule(p, 'allow', new Belonger(), 'join', 'party')
      ).toStrictEqual([map()]);
    });

    test('can handle variables as arguments', async () => {
      const p = new Polar();
      await p.loadFile(await tempFileFx());
      expect(await queryRule(p, 'f', new Variable('a'))).toStrictEqual([
        map({ a: 1 }),
        map({ a: 2 }),
        map({ a: 3 }),
      ]);
    });
  });
});

describe('#makeInstance', () => {
  test('handles no args', async () => {
    const p = new Polar();
    p.registerClass(ConstructorNoArgs);
    await p.__host().makeInstance(ConstructorNoArgs.name, [], 1);
    const instance = p.__host().getInstance(1);
    expect(instance).toStrictEqual(new ConstructorNoArgs());
  });

  test('handles positional args', async () => {
    const p = new Polar();
    p.registerClass(ConstructorArgs);
    const one = p.__host().toPolar(1);
    const two = p.__host().toPolar(2);
    await p.__host().makeInstance(ConstructorArgs.name, [one, two], 1);
    const instance = p.__host().getInstance(1);
    expect(instance).toStrictEqual(new ConstructorArgs(1, 2));
  });

  test('rejects keyword args', () => {
    const p = new Polar();
    p.registerClass(ConstructorArgs);
    const q = 'x = new ConstructorArgs(first: 1, second: 2)';
    expect(query(p, q)).rejects.toThrow(KwargsError);
  });
});

// test_nil
describe('null is pre-registered', () => {
  test('as nil', async () => {
    const p = new Polar();
    await p.loadStr('null(nil);');
    expect(await qvar(p, 'null(x)', 'x')).toStrictEqual([null]);
    expect(await queryRule(p, 'null', [])).toStrictEqual([]);
  });
});

describe('#registerConstant', () => {
  test('works', async () => {
    const p = new Polar();
    const d = { a: 1 };
    p.registerConstant(d, 'd');
    expect(await qvar(p, 'd.a = x', 'x')).toStrictEqual([1]);
  });

  describe('can call host language methods', () => {
    test('on strings', async () => {
      const p = new Polar();
      expect(
        await query(p, 'x = "abc" and x.indexOf("bc") = 1')
      ).toStrictEqual([map({ x: 'abc' })]);
    });

    test('on numbers', async () => {
      const p = new Polar();
      expect(
        await query(p, 'f = 314.159 and f.toExponential() = "3.14159e+2"')
      ).toStrictEqual([map({ f: 314.159 })]);
    });

    test('on lists', async () => {
      const p = new Polar();
      expect(
        await query(
          p,
          'l = [1, 2, 3] and l.indexOf(3) = 2 and l.concat([4]) = [1, 2, 3, 4]'
        )
      ).toStrictEqual([map({ l: [1, 2, 3] })]);
    });

    test('on dicts', async () => {
      const p = new Polar();
      expect(
        await query(p, 'd = {a: 1} and d.a = 1 and d.hasOwnProperty("a")')
      ).toStrictEqual([map({ d: { a: 1 } })]);
    });

    describe('that return undefined', () => {
      test('without things blowing up', async () => {
        const p = new Polar();
        p.registerConstant({}, 'u');
        expect(await query(p, 'u.x = u.y')).toStrictEqual([map()]);
        expect(query(p, 'u.x.y')).rejects.toThrow();
      });
    });

    // test_host_method_nil
    test('that return null', async () => {
      const p = new Polar();
      p.registerConstant({ x: null }, 'u');
      expect(await query(p, 'u.x = nil')).toStrictEqual([map()]);
      expect(query(p, 'u.x.y')).rejects.toThrow();
    });
  });

  // TODO(gj): Is this expected?
  test('errors when calling host language methods on booleans', () => {
    const p = new Polar();
    expect(query(p, 'b = true and b.constructor = Boolean')).rejects.toThrow(
      'Type error: can only perform lookups on dicts and instances, this is true at line 1, column 5'
    );
  });

  test('registering the same constant twice overwrites', () => {
    const p = new Polar();
    p.registerConstant(1, 'x');
    p.registerConstant(2, 'x');
    expect(p.loadStr('?= x == 2;')).resolves;
  });
});

describe('unifying a promise', () => {
  describe('with another promise', () => {
    test('succeeds if the resolved values unify', async () => {
      const p = new Polar();
      const a = Promise.resolve(1);
      const b = Promise.resolve(1);
      p.registerConstant(a, 'a');
      p.registerConstant(b, 'b');
      const result = await query(p, 'a = b');
      expect(result).toStrictEqual([map()]);
    });

    test("fails if the resolved values don't unify", async () => {
      const p = new Polar();
      const a = Promise.resolve(1);
      const b = Promise.resolve(2);
      p.registerConstant(a, 'a');
      p.registerConstant(b, 'b');
      const result = await query(p, 'a = b');
      expect(result).toStrictEqual([]);
    });
  });

  describe('with a non-promise', () => {
    // TODO(gj): Un-skip when external instances can unify with non-external instances.
    xtest('succeeds if the resolved value unifies with the non-promise', async () => {
      const p = new Polar();
      const a = Promise.resolve(1);
      p.registerConstant(a, 'a');
      const result = await query(p, 'a = 1');
      expect(result).toStrictEqual([map()]);
    });

    test("fails if the resolved value doesn't unify with the non-promise", async () => {
      const p = new Polar();
      const a = Promise.resolve(1);
      p.registerConstant(a, 'a');
      const result = await query(p, 'a = 2');
      expect(result).toStrictEqual([]);
    });
  });
});

describe('errors', () => {
  describe('with inline queries', () => {
    test('succeeds if all inline queries succeed', () => {
      const p = new Polar();
      expect(p.loadStr('f(1); f(2); ?= f(1); ?= not f(3);')).resolves;
    });

    test('fails if an inline query fails', () => {
      const p = new Polar();
      expect(p.loadStr('g(1); ?= g(2);')).rejects.toThrow(
        InlineQueryFailedError
      );
    });
  });

  describe('with expressions', () => {
    test('errors if an expression is received', () => {
      const p = new Polar();
      expect(p.loadStr('f(x) if x > 2;')).resolves;
      let result = p.query('f(x)').next();
      expect(result).rejects.toThrow(UnexpectedPolarTypeError);
    });
  });

  describe('when parsing', () => {
    test('raises on IntegerOverflow errors', () => {
      const p = new Polar();
      const int = '18446744073709551616';
      const rule = `f(a) if a = ${int};`;
      expect(p.loadStr(rule)).rejects.toThrow(
        `'${int}' caused an integer overflow at line 1, column 13`
      );
    });

    test('raises on InvalidTokenCharacter errors', () => {
      const p = new Polar();
      const rule = `
        f(a) if a = "this is not
        allowed";
      `;
      expect(p.loadStr(rule)).rejects.toThrow(
        "'\\n' is not a valid character. Found in this is not at line 2, column 33"
      );
    });

    test.todo('raises on InvalidToken');

    test('raises on UnrecognizedEOF errors', () => {
      const p = new Polar();
      const rule = 'f(a)';
      expect(p.loadStr(rule)).rejects.toThrow(
        'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 5'
      );
    });

    test('raises on UnrecognizedToken errors', () => {
      const p = new Polar();
      const rule = '1;';
      expect(p.loadStr(rule)).rejects.toThrow(
        "did not expect to find the token '1' at line 1, column 1"
      );
    });

    test.todo('raises on ExtraToken');
  });

  describe('runtime errors', () => {
    test('include a stack trace', async () => {
      const p = new Polar();
      await p.loadStr('foo(a,b) if a in b;');
      expect(query(p, 'foo(1,2)')).rejects.toThrow(
        `trace (most recent evaluation last):
  in query at line 1, column 1
    foo(1,2)
  in rule foo at line 1, column 13
    a in b
Type error: can only use \`in\` on an iterable value, this is Number(Integer(2)) at line 1, column 7`
      );
    });

    test('work for lookups', () => {
      const p = new Polar();
      p.registerConstant(undefined, 'undefined');
      expect(query(p, 'undefined.foo')).rejects.toThrow(
        `trace (most recent evaluation last):
  in query at line 1, column 1
    undefined.foo
  in query at line 1, column 1
    undefined.foo
Application error: Cannot read property 'foo' of undefined at line 1, column 1`
      );
    });
  });
});

describe('unbound variables', () => {
  test('returns unbound properly', async () => {
    const p = new Polar();
    await p.loadStr('rule(_x, y) if y = 1;');

    const result = (await query(p, 'rule(x, y)'))[0];

    expect(result.get('y')).toBe(1);
    expect(result.get('x')).toBeInstanceOf(Variable);
  });
});

describe('±∞ and NaN', () => {
  test('are handled properly', async () => {
    const p = new Polar();
    p.registerConstant(NaN, 'nan');
    p.registerConstant(Infinity, 'inf');
    p.registerConstant(-Infinity, 'neg_inf');

    const nan = (await query(p, 'x = nan'))[0];
    expect(Number.isNaN(nan.get('x'))).toBe(true);
    expect(await query(p, 'nan = nan')).toStrictEqual([]);

    const inf = (await query(p, 'x = inf'))[0];
    expect(inf.get('x')).toEqual(Infinity);
    expect(await query(p, 'inf = inf')).toStrictEqual([map()]);

    const negInf = (await query(p, 'x = neg_inf'))[0];
    expect(negInf.get('x')).toEqual(-Infinity);
    expect(await query(p, 'neg_inf = neg_inf')).toStrictEqual([map()]);

    expect(await query(p, 'inf = neg_inf')).toStrictEqual([]);
    expect(await query(p, 'inf < neg_inf')).toStrictEqual([]);
    expect(await query(p, 'neg_inf < inf')).toStrictEqual([map()]);
  });
});

test('ExternalOp events test for equality succeeds', async () => {
  // js objects are never equal so we override
  // weirdness in js definition of equality
  const p = new Polar({ equalityFn: (_x, _y) => true });
  p.registerClass(X);
  expect(await query(p, 'new X() == new X()')).toStrictEqual([map()]);
  expect(await query(p, 'new X() != new X()')).toStrictEqual([]);
});

describe('iterators', () => {
  test('work over builtins', async () => {
    const p = new Polar();
    expect(
      await qvar(
        p,
        'd = {a: 1, b: 2} and x in Dictionary.entries({a: 1, b: 2}) and x in d',
        'x'
      )
    ).toStrictEqual([
      ['a', 1],
      ['b', 2],
    ]);
  });

  test('fails for non iterables', async () => {
    const p = new Polar();
    p.registerClass(NonIterable, 'NonIterable');
    expect(query(p, 'x in new NonIterable()')).rejects.toThrow(
      InvalidIteratorError
    );
  });

  test('work for custom classes', async () => {
    const p = new Polar();
    p.registerClass(BarIterator, 'BarIterator');
    expect(
      await qvar(p, 'x in new BarIterator([1, 2, 3])', 'x')
    ).toStrictEqual([1, 2, 3]);
    expect(
      await qvar(p, 'x = new BarIterator([1, 2, 3]).sum()', 'x', true)
    ).toBe(6);
  });
});
