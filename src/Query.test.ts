import { Polar } from './Polar';
import { query, qvar } from '../test/helpers';

describe('#registerCall', () => {
  test('synchronous values', async () => {
    const sync = {
      undefined: undefined,
      null: null,
      generator: function* () {
        yield 1;
        yield 2;
        yield 3;
      },
      function: function () {
        return [1, 2, 3];
      },
      arrowFn: () => [1, 2, 3],
      iterator: [1, 2, 3].values(),
      arrowFnReturningIterator: () => [1, 2, 3].values(),
      array: [1, 2, 3],
      object: { x: [1, 2, 3] },
      invalidCustomIterator: { next: () => ({ value: 1 }) },
      validCustomIterator: {
        next: function () {
          return this.i > 3 ? { done: true } : { value: this.i++ };
        },
        i: 1,
        [Symbol.iterator]: function () {
          return this;
        },
      },
      map: new Map([
        [1, 1],
        [2, 2],
        [3, 3],
      ]),
      set: new Set([1, 2, 3]),
    };

    const p = new Polar();
    p.registerConstant(sync, 'sync');

    expect(await qvar(p, 'sync.undefined = x', 'x', true)).toBeUndefined();
    await expect(query(p, 'sync.attribute_undefined = x')).rejects.toThrow(
      'attribute_undefined not found on'
    );
    await expect(query(p, 'sync.undefined()')).rejects.toThrow(
      '.undefined is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'sync.null = x', 'x', true)).toBeNull();
    await expect(query(p, 'sync.null()')).rejects.toThrow(
      '.null is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'sync.generator = x', 'x', true)).toStrictEqual(
      sync.generator
    );
    expect(await qvar(p, 'x in sync.generator()', 'x')).toStrictEqual([
      1, 2, 3,
    ]);

    expect(await qvar(p, 'sync.function = x', 'x', true)).toStrictEqual(
      sync.function
    );
    expect(await qvar(p, 'x = sync.function()', 'x')).toStrictEqual([
      [1, 2, 3],
    ]);

    expect(await qvar(p, 'sync.arrowFn = x', 'x', true)).toStrictEqual(
      sync.arrowFn
    );
    expect(await qvar(p, 'sync.arrowFn() = x', 'x')).toStrictEqual([[1, 2, 3]]);

    expect(await qvar(p, 'x in sync.iterator', 'x')).toStrictEqual([1, 2, 3]);
    await expect(query(p, 'sync.iterator()')).rejects.toThrow(
      '.iterator is not a function at line 1, column 1'
    );

    expect(
      await qvar(p, 'x = sync.arrowFnReturningIterator', 'x', true)
    ).toStrictEqual(sync.arrowFnReturningIterator);
    expect(
      await qvar(p, 'x in sync.arrowFnReturningIterator()', 'x')
    ).toStrictEqual([1, 2, 3]);

    expect(await qvar(p, 'sync.array = x', 'x', true)).toStrictEqual(
      sync.array
    );
    expect(await qvar(p, 'x in sync.array', 'x')).toStrictEqual(sync.array);
    await expect(query(p, 'sync.array()')).rejects.toThrow(
      '.array is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'sync.object = x', 'x')).toStrictEqual([sync.object]);
    await expect(query(p, 'sync.object()')).rejects.toThrow(
      '.object is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'x in sync.validCustomIterator', 'x')).toStrictEqual([
      1, 2, 3,
    ]);
    await expect(query(p, 'sync.validCustomIterator()')).rejects.toThrow(
      '.validCustomIterator is not a function at line 1, column 1'
    );

    expect(
      await qvar(p, 'sync.invalidCustomIterator = x', 'x', true)
    ).toStrictEqual(sync.invalidCustomIterator);
    await expect(query(p, 'sync.invalidCustomIterator()')).rejects.toThrow(
      '.invalidCustomIterator is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'sync.map = x', 'x', true)).toStrictEqual(sync.map);
    await expect(query(p, 'sync.map()')).rejects.toThrow(
      '.map is not a function at line 1, column 1'
    );

    expect(await qvar(p, 'sync.set = x', 'x', true)).toStrictEqual(sync.set);
    await expect(query(p, 'sync.set()')).rejects.toThrow(
      '.set is not a function at line 1, column 1'
    );
  });

  test('asynchronous values', async () => {
    const async = {
      promise: Promise.resolve([1, 2, 3]),
      function: function () {
        return Promise.resolve([1, 2, 3]);
      },
      arrow: () => Promise.resolve([1, 2, 3]),
      generator: async function* () {
        yield await Promise.resolve(1);
        yield await Promise.resolve(2);
        yield await Promise.resolve(3);
      },
      iterator: {
        [Symbol.asyncIterator]() {
          return {
            i: 1,
            next() {
              return Promise.resolve(
                this.i < 4 ? { value: this.i++ } : { done: true }
              );
            },
          };
        },
      },
    };

    const p = new Polar();
    p.registerConstant(async, 'async');

    expect(await qvar(p, 'async.promise = x', 'x', true)).toStrictEqual([
      1, 2, 3,
    ]);

    expect(await qvar(p, 'async.function() = x', 'x', true)).toStrictEqual([
      1, 2, 3,
    ]);

    expect(await qvar(p, 'async.arrow() = x', 'x', true)).toStrictEqual([
      1, 2, 3,
    ]);

    expect(await qvar(p, 'x in async.generator()', 'x')).toStrictEqual([
      1, 2, 3,
    ]);

    expect(await qvar(p, 'x in async.iterator', 'x')).toStrictEqual([1, 2, 3]);
  });
});
