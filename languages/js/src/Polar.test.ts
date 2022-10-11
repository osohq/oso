import { Polar } from './Polar';
import { Expression } from './Expression';
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
} from '../test/helpers';
import {
  A,
  BaseActor,
  Animal,
  B,
  Bar,
  BarIterator,
  Belonger,
  C,
  ConstructorArgs,
  ConstructorNoArgs,
  ConstructorMapObjectArgs,
  Counter,
  Foo,
  NonIterable,
  User,
  Widget,
  X,
  ConstructorAnyArg,
} from '../test/classes';
import {
  DuplicateClassAliasError,
  InlineQueryFailedError,
  InvalidConstructorError,
  KwargsError,
  PolarFileNotFoundError,
  PolarFileExtensionError,
  InvalidIteratorError,
  UnexpectedExpressionError,
} from './errors';
import * as rolesHelpers from '../test/rolesHelpers';
import { Dict } from './types';

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
    expect(() => p.registerClass(BaseActor)).not.toThrow();
    expect(() => p.registerClass(BaseActor)).toThrow(DuplicateClassAliasError);
  });

  test('errors when registering the same alias twice', () => {
    const p = new Polar();
    expect(() => p.registerClass(BaseActor)).not.toThrow();
    expect(() => p.registerClass(User, { name: 'BaseActor' })).toThrow(
      DuplicateClassAliasError
    );
  });

  test('can register the same class under different aliases', async () => {
    const p = new Polar();
    p.registerClass(A, { name: 'A' });
    p.registerClass(A, { name: 'B' });
    expect(await query(p, 'new A().a() = new B().a()')).toStrictEqual([map()]);
  });

  test('registers a JS class with Polar', async () => {
    const p = new Polar();
    p.registerClass(Foo);
    p.registerClass(Bar);
    expect(await qvar(p, 'new Foo("A").a = x', 'x', true)).toStrictEqual('A');
    await expect(qvar(p, 'new Foo("A").a() = x', 'x', true)).rejects.toThrow(
      `trace (most recent evaluation last):
  002: new Foo("A")
    in query at line 1, column 1
  001: new Foo("A").a()
    in query at line 1, column 1
  000: new Foo("A").a()
    in query at line 1, column 1

Application error: Foo { a: 'A' }.a is not a function at line 1, column 1`
    );
    await expect(qvar(p, 'x in new Foo("A").b', 'x', true)).rejects.toThrow(
      "'function' is not iterable"
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

  test('rejects keyword arguments in method calls', async () => {
    const p = new Polar();
    p.registerClass(A);
    await expect(query(p, 'x = (new A()).a(arg: 1)')).rejects.toThrow(
      KwargsError
    );
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
      const p = new Polar({
        // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
        equalityFn: (x: any, y: any) => x.family === y.family,
      });
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
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      p.registerClass(Math);
    }).toThrow(InvalidConstructorError);
  });
});

describe('conversions between JS + Polar values', () => {
  test('returns JS instances from external calls', async () => {
    const actor = new BaseActor('sam');
    const widget = new Widget('1');
    const p = new Polar();
    await p.loadStr(
      'allow(actor, _action, resource) if actor.widget().id = resource.id;'
    );
    const result = await queryRule(p, 'allow', actor, 'read', widget);
    expect(result).toStrictEqual([map()]);
  });

  test('unifies equivalent JS and Polar types', async () => {
    const p = new Polar();
    let result = await query(p, 'new Integer(1) = 1');
    expect(result).toStrictEqual([map()]);
    result = await query(p, 'new String("foo") = "foo"');
    expect(result).toStrictEqual([map()]);
  });

  test('handles Generator external call results', async () => {
    const actor = new BaseActor('sam');
    const p = new Polar();
    await p.loadStr('widgets(actor, x) if w in actor.widgets() and x = w.id;');
    const result = await queryRule(p, 'widgets', actor, new Variable('x'));
    expect(result).toStrictEqual([map({ x: '2' }), map({ x: '3' })]);
  });

  describe('caches instances and does not leak them', () => {
    test("instances created in a query don't outlive the query", async () => {
      const p = new Polar();
      p.registerClass(Counter);

      const preLoadInstanceCount = p.getHost().instances().length;
      await p.loadStr('f(_: Counter) if Counter.count() > 0;');
      const preQueryInstanceCount = p.getHost().instances().length;
      expect(preLoadInstanceCount).toStrictEqual(preQueryInstanceCount);

      expect(Counter.count()).toBe(0);
      const c = new Counter();
      expect(Counter.count()).toBe(1);

      expect(await queryRule(p, 'f', c)).toStrictEqual([map()]);
      const postQueryInstanceCount = p.getHost().instances().length;
      expect(preQueryInstanceCount).toStrictEqual(postQueryInstanceCount);

      expect(Counter.count()).toBe(1);
    });
  });
});

describe('#loadFiles', () => {
  test('loads a Polar file', async () => {
    const p = new Polar();
    await p.loadFiles([await tempFileFx()]);
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
  });

  test('passes the filename across the FFI boundary', async () => {
    const p = new Polar();
    const file = await tempFile(';', 'invalid.polar');
    await expect(p.loadFiles([file])).rejects.toThrow(
      `did not expect to find the token ';' at line 1, column 1 of file ${file}`
    );
  });

  test('throws if given a non-Polar file', async () => {
    const p = new Polar();
    await expect(p.loadFiles(['other.ext'])).rejects.toThrow(
      PolarFileExtensionError
    );
  });

  test('throws if given a non-existent file', async () => {
    const p = new Polar();
    await expect(p.loadFiles(['other.polar'])).rejects.toThrow(
      PolarFileNotFoundError
    );
  });

  test('throws if two files with the same contents are loaded', async () => {
    const p = new Polar();
    await expect(
      p.loadFiles([
        await tempFile('', 'a.polar'),
        await tempFile('', 'b.polar'),
      ])
    ).rejects.toThrow(
      /Problem loading file: A file with the same contents as .*b.polar named .*a.polar has already been loaded./
    );
  });

  // TODO(gj): This is no longer possible but might again become possible if we
  // add a `loadStrings()` method that accepts `{contents, filename}` tuples
  // from the user. However, we could also have this hypothetical
  // `loadStrings()` method only accept `contents` and avoid the issue.
  xtest('throws if two files with the same name are loaded', async () => {
    const p = new Polar();
    const filename1 = await tempFile('f();', 'a.polar');
    const filename2 = await tempFile('g();', 'a.polar');
    await expect(p.loadFiles([filename1, filename2])).rejects.toThrow(
      /Problem loading file: A file with the name .*a.polar, but different contents has already been loaded./
    );
  });

  // test_load_multiple_files_same_name_different_path
  test('can load two files with the same name but different paths', async () => {
    const p = new Polar();
    const filename1 = await tempFile('f(1);f(2);f(3);', 'a.polar');
    const filename2 = await tempFile('g(1);g(2);g(3);', 'other/a.polar');
    await expect(p.loadFiles([filename1, filename2])).resolves.not.toThrow();
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    expect(await qvar(p, 'g(x)', 'x')).toStrictEqual([1, 2, 3]);
  });

  test('throws if the same file is loaded twice', async () => {
    const p = new Polar();
    const file = await tempFileFx();
    await expect(p.loadFiles([file, file])).rejects.toThrow(
      /Problem loading file: File .*f.polar has already been loaded./
    );
  });

  test('can load multiple files', async () => {
    const p = new Polar();
    await p.loadFiles([await tempFileFx(), await tempFileGx()]);
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    expect(await qvar(p, 'g(x)', 'x')).toStrictEqual([1, 2, 3]);
  });
});

describe('#clearRules', () => {
  test('clears the KB', async () => {
    const p = new Polar();
    await p.loadFiles([await tempFileFx()]);
    expect(await qvar(p, 'f(x)', 'x')).toStrictEqual([1, 2, 3]);
    p.clearRules();
    await expect(query(p, 'f(x)')).rejects.toThrow(
      'Query for undefined rule `f`'
    );
  });

  test('does not clear registered classes', async () => {
    const p = new Polar();
    p.registerClass(Belonger, { name: 'BaseActor' });
    p.clearRules();
    expect(await query(p, 'x = new BaseActor()')).toHaveLength(1);
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
      p.registerClass(Belonger, { name: 'BaseActor' });
      await p.loadStr(
        'allow(actor: BaseActor, "join", "party") if "social" in actor.groups();'
      );
      expect(
        await queryRule(p, 'allow', new Belonger(), 'join', 'party')
      ).toStrictEqual([map()]);
    });

    test('can handle variables as arguments', async () => {
      const p = new Polar();
      await p.loadFiles([await tempFileFx()]);
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
    await p.getHost().makeInstance(ConstructorNoArgs.name, [], 1);
    const instance = p.getHost().getInstance(1);
    expect(instance).toStrictEqual(new ConstructorNoArgs());
  });

  test('handles positional args', async () => {
    const p = new Polar();
    p.registerClass(ConstructorArgs);
    const one = p.getHost().toPolar(1);
    const two = p.getHost().toPolar(2);
    await p.getHost().makeInstance(ConstructorArgs.name, [one, two], 1);
    const instance = p.getHost().getInstance(1);
    expect(instance).toStrictEqual(new ConstructorArgs(1, 2));
  });

  test('handles JS Maps & Polar dicts', async () => {
    const p = new Polar();
    p.registerClass(ConstructorMapObjectArgs);
    p.registerClass(ConstructorAnyArg);
    p.registerClass(Map);
    const shouldPass = [
      // All args match ctor's expectation.
      '?= x = new ConstructorMapObjectArgs(new Map([["one", 1]]), {two: 2}, new Map([["three", 3]]), {four: 4}) and x.one = 1 and x.two = 2 and x.three = 3 and x.four = 4;',
      // All Maps passed instead of dicts. Field lookups on Maps return undefined.
      '?= x = new ConstructorMapObjectArgs(new Map([["one", 1]]), new Map([["two", 2]]), new Map([["three", 3]]), new Map([["four", 4]])) and x.one = 1 and x.two = undefined and x.three = 3 and x.four = undefined;',
      '?= new ConstructorAnyArg({x: 1}).opts.x = 1;',
    ];
    await expect(
      Promise.all(shouldPass.map(x => p.loadStr(x)))
    ).resolves.not.toThrow();

    // All dicts passed instead of Maps. TypeErrors abound when we try to
    // call Map methods on the dicts.
    await expect(
      p.loadStr(
        '?= new ConstructorMapObjectArgs({one: 1}, {two: 2}, {three: 3}, {four: 4});'
      )
    ).rejects.toThrow(TypeError('oneMap.get is not a function'));
  });

  test('rejects keyword args', async () => {
    const p = new Polar();
    p.registerClass(ConstructorArgs);
    const q = 'x = new ConstructorArgs(first: 1, second: 2)';
    await expect(query(p, q)).rejects.toThrow(KwargsError);
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
      expect(await query(p, 'x = "abc" and x.indexOf("bc") = 1')).toStrictEqual(
        [map({ x: 'abc' })]
      );
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
        p.registerConstant({ x: undefined, y: undefined }, 'u');
        expect(await query(p, 'u.x = u.y')).toStrictEqual([map()]);
        await expect(query(p, 'u.x.y')).rejects.toThrow();
      });
    });

    // test_host_method_nil
    test('that return null', async () => {
      const p = new Polar();
      p.registerConstant({ x: null }, 'u');
      expect(await query(p, 'u.x = nil')).toStrictEqual([map()]);
      await expect(query(p, 'u.x.y')).rejects.toThrow();
    });
  });

  // TODO(gj): Is this expected?
  test('errors when calling host language methods on booleans', async () => {
    const p = new Polar();
    await expect(
      query(p, 'b = true and b.constructor = Boolean')
    ).rejects.toThrow(
      'Type error: can only perform lookups on dicts and instances, this is true at line 1, column 5'
    );
  });

  test('registering the same constant twice overwrites', async () => {
    const p = new Polar();
    p.registerConstant(1, 'x');
    p.registerConstant(2, 'x');
    await expect(p.loadStr('?= x == 2;')).resolves.not.toThrow();
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
    test('succeeds if all inline queries succeed', async () => {
      const p = new Polar();
      await expect(
        p.loadStr('f(1); f(2); ?= f(1); ?= not f(3);')
      ).resolves.not.toThrow();
    });

    test('fails if an inline query fails', async () => {
      const p = new Polar();
      await expect(p.loadStr('g(1); ?= g(2);')).rejects.toThrow(
        InlineQueryFailedError
      );
    });
  });

  describe('when parsing', () => {
    test('raises on IntegerOverflow errors', async () => {
      const p = new Polar();
      const int = '18446744073709551616';
      const rule = `f(a) if a = ${int};`;
      await expect(p.loadStr(rule)).rejects.toThrow(
        `'${int}' caused an integer overflow at line 1, column 13`
      );
    });

    test('raises on InvalidTokenCharacter errors', async () => {
      const p = new Polar();
      const rule = `
        f(a) if a = "this is not
        allowed";
      `;
      await expect(p.loadStr(rule)).rejects.toThrow(
        "'\\n' is not a valid character. Found in this is not at line 2, column 33"
      );
    });

    test.todo('raises on InvalidToken');

    test('raises on UnrecognizedEOF errors', async () => {
      const p = new Polar();
      const rule = 'f(a)';
      await expect(p.loadStr(rule)).rejects.toThrow(
        'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 5'
      );
    });

    test('raises on UnrecognizedToken errors', async () => {
      const p = new Polar();
      const rule = '1;';
      await expect(p.loadStr(rule)).rejects.toThrow(
        "did not expect to find the token '1' at line 1, column 1"
      );
    });

    test.todo('raises on ExtraToken');
  });

  describe('runtime errors', () => {
    test('include a stack trace', async () => {
      const p = new Polar();
      await p.loadStr('foo(a,b) if a in b;');
      await expect(query(p, 'foo(1,2)')).rejects.toThrow(
        `trace (most recent evaluation last):
  002: foo(1,2)
    in query at line 1, column 1
  001: a in b
    in rule foo at line 1, column 13

Type error: can only use \`in\` on an iterable value, this is Number(Integer(2)) at line 1, column 7`
      );
    });

    test('work for lookups', async () => {
      const p = new Polar();
      p.registerConstant(undefined, 'undefined');
      await expect(query(p, 'undefined.foo')).rejects.toThrow(
        `trace (most recent evaluation last):
  001: undefined.foo
    in query at line 1, column 1
  000: undefined.foo
    in query at line 1, column 1

Application error: foo not found on undefined`
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
  const p = new Polar({ equalityFn: () => true });
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
    p.registerClass(NonIterable);
    await expect(query(p, 'x in new NonIterable()')).rejects.toThrow(
      InvalidIteratorError
    );
  });

  test('work for custom classes', async () => {
    const p = new Polar();
    p.registerClass(BarIterator);
    expect(await qvar(p, 'x in new BarIterator([1, 2, 3])', 'x')).toStrictEqual(
      [1, 2, 3]
    );
    expect(
      await qvar(p, 'x = new BarIterator([1, 2, 3]).sum()', 'x', true)
    ).toBe(6);
  });
});

test('handles expressions', async () => {
  const p = new Polar();
  await p.loadStr('f(x) if x > 2;');
  const result = (await query(p, 'f(x)', { acceptExpression: true }))[0];
  const x = result.get('x');
  expect(x).toBeInstanceOf(Expression);
  const gt = new Expression('Gt', [new Variable('_this'), 2]);
  const expected = new Expression('And', [gt]);
  expect(x).toStrictEqual(expected);
});

test("errors on expressions when acceptExpression isn't set", async () => {
  const p = new Polar();
  await p.loadStr('f(x) if x > 2;');
  const result = query(p, 'f(x)');
  await expect(result).rejects.toThrow(UnexpectedExpressionError);
});

// test_roles_integration
describe('Oso Roles', () => {
  test('works', async () => {
    const { Issue, Org, Repo, Role, User } = rolesHelpers;
    // Test fixtures.
    const osohq = new Org('osohq');
    const apple = new Org('apple');
    const oso = new Repo('oso', osohq);
    const ios = new Repo('ios', apple);
    const bug = new Issue('bug', oso);
    const laggy = new Issue('laggy', ios);

    const osohqOwner = new Role('owner', osohq);
    const osohqMember = new Role('member', osohq);

    const leina = new User('leina', [osohqOwner]);
    const steve = new User('steve', [osohqMember]);

    const policy = `
      allow(actor, action, resource) if
        has_permission(actor, action, resource);

      has_role(user: User, name: String, resource: Resource) if
        role in user.roles and
        role matches { name: name, resource: resource };

      actor User {}

      resource Org {
        roles = [ "owner", "member" ];
        permissions = [ "invite", "create_repo" ];

        "create_repo" if "member";
        "invite" if "owner";

        "member" if "owner";
      }

      resource Repo {
        roles = [ "writer", "reader" ];
        permissions = [ "push", "pull" ];
        relations = { parent: Org };

        "pull" if "reader";
        "push" if "writer";

        "reader" if "writer";

        "reader" if "member" on "parent";
        "writer" if "owner" on "parent";
      }

      has_relation(org: Org, "parent", repo: Repo) if
        org = repo.org;

      resource Issue {
        permissions = [ "edit" ];
        relations = { parent: Repo };

        "edit" if "writer" on "parent";
      }

      has_relation(repo: Repo, "parent", issue: Issue) if
        repo = issue.repo;
    `;

    const p = new Polar();
    [Org, Repo, Issue, User].forEach(c => p.registerClass(c));
    await p.loadStr(policy);

    const isAllowed = async (...args: unknown[]) => {
      const result = await query(p, pred('allow', ...args));
      return result.length !== 0;
    };

    expect(await isAllowed(leina, 'invite', osohq));
    expect(await isAllowed(leina, 'create_repo', osohq));
    expect(await isAllowed(leina, 'push', oso));
    expect(await isAllowed(leina, 'pull', oso));
    expect(await isAllowed(leina, 'edit', bug));

    expect(!(await isAllowed(steve, 'invite', osohq)));
    expect(await isAllowed(steve, 'create_repo', osohq));
    expect(!(await isAllowed(steve, 'push', oso)));
    expect(await isAllowed(steve, 'pull', oso));
    expect(!(await isAllowed(steve, 'edit', bug)));

    expect(!(await isAllowed(leina, 'edit', laggy)));
    expect(!(await isAllowed(steve, 'edit', laggy)));

    let gabe = new User('gabe', []);
    expect(!(await isAllowed(gabe, 'edit', bug)));
    gabe = new User('gabe', [osohqMember]);
    expect(!(await isAllowed(gabe, 'edit', bug)));
    gabe = new User('gabe', [osohqOwner]);
    expect(await isAllowed(gabe, 'edit', bug));
  });

  test('rule types correctly check subclasses', async () => {
    class Foo {}
    class Bar extends Foo {}
    class Baz extends Bar {}
    class Bad {}

    // NOTE: keep this order of registering classes--confirms that MROs are added at the correct time
    const p = new Polar();
    p.registerClass(Baz);
    p.registerClass(Bar);
    p.registerClass(Foo);
    p.registerClass(Bad);

    const policy = `type f(_x: Integer);
                    f(1);`;
    await p.loadStr(policy);
    p.clearRules();

    const policy2 =
      policy +
      `type f(_x: Foo);
       type f(_x: Foo, _y: Bar);
       f(_x: Bar);
       f(_x: Baz);`;
    await p.loadStr(policy2);
    p.clearRules();

    const policy3 = policy2 + 'f(_x: Bad);';
    await expect(p.loadStr(policy3)).rejects.toThrow('Invalid rule');

    // Test with fields
    const policy4 = `type f(_x: Foo{id: 1});
                     f(_x: Bar{id: 1});
                     f(_x: Baz{id: 1});`;
    await p.loadStr(policy4);
    p.clearRules();

    await expect(p.loadStr(policy4 + 'f(_x: Baz);')).rejects.toThrow(
      'Invalid rule'
    );

    // Test invalid rule type
    const policy5 = policy4 + 'type f(x: Foo, x.baz);';
    await expect(p.loadStr(policy5)).rejects.toThrow('Invalid rule type');
  });
});

test('can specialize on a dict with undefineds', async () => {
  const p = new Polar();
  await p.loadStr('f(_: {x: 1});');

  const noAttr = {};
  const hasAttr = { x: 1 };

  const result1 = await query(p, pred('f', hasAttr));
  expect(result1).toStrictEqual([map()]);

  const result2 = await query(p, pred('f', noAttr));
  expect(result2).toStrictEqual([]);

  Object.setPrototypeOf(noAttr, hasAttr);

  const result3 = await query(p, pred('f', noAttr));
  expect(result3).toStrictEqual([map()]);
});

test('can specialize with custom `isa` check', async () => {
  const p = new Polar();
  class Foo {}
  class Bar {}
  p.registerClass(Foo);
  p.registerClass(Bar, {
    isaCheck: instance =>
      instance instanceof Object &&
      !!instance.typename && // eslint-disable-line @typescript-eslint/no-unsafe-member-access
      instance.typename === 'Bar', // eslint-disable-line @typescript-eslint/no-unsafe-member-access
  });

  const foo = new Foo();
  const bar = { typename: 'Bar' };
  const dict = new Dict({});
  const neither = { typename: 'Foo' };

  await p.loadStr('is_foo(_: Foo); is_bar(_: Bar); is_dict(_: Dictionary);');

  const result1 = await query(p, pred('is_foo', foo));
  expect(result1).toStrictEqual([map()]);

  const result2 = await query(p, pred('is_foo', bar));
  expect(result2).toStrictEqual([]);

  const result3 = await query(p, pred('is_foo', neither));
  expect(result3).toStrictEqual([]);

  const result4 = await query(p, pred('is_bar', foo));
  expect(result4).toStrictEqual([]);

  const result5 = await query(p, pred('is_bar', bar));
  expect(result5).toStrictEqual([map()]);

  const result6 = await query(p, pred('is_bar', neither));
  expect(result6).toStrictEqual([]);

  const result7 = await query(p, pred('is_dict', foo));
  expect(result7).toStrictEqual([]);

  const result8 = await query(p, pred('is_dict', bar));
  expect(result8).toStrictEqual([]);

  const result9 = await query(p, pred('is_dict', neither));
  expect(result9).toStrictEqual([]);

  const result10 = await query(p, pred('is_dict', dict));
  expect(result10).toStrictEqual([map()]);
});
