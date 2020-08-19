import { Polar } from './Polar';
import { query, qvar } from '../test/helpers';

test('#registerCall', () => {
  const f = {
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
        return this._i > 3 ? { done: true } : { value: this._i++ };
      },
      _i: 1,
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
  p.registerConstant('f', f);

  expect(qvar(p, 'f.undefined = x', 'x', true)).toBeUndefined();
  expect(() => query(p, 'f.undefined()')).toThrow(
    '.undefined is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.null = x', 'x', true)).toBeNull();
  expect(() => query(p, 'f.null()')).toThrow(
    '.null is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.generator = x', 'x', true)).toStrictEqual(f.generator);
  expect(qvar(p, 'f.generator() = x', 'x')).toStrictEqual([1, 2, 3]);

  expect(qvar(p, 'f.function = x', 'x', true)).toStrictEqual(f.function);
  expect(qvar(p, 'f.function() = x', 'x')).toStrictEqual([[1, 2, 3]]);

  expect(qvar(p, 'f.arrowFn = x', 'x', true)).toStrictEqual(f.arrowFn);
  expect(qvar(p, 'f.arrowFn() = x', 'x')).toStrictEqual([[1, 2, 3]]);

  expect(qvar(p, 'f.iterator = x', 'x')).toStrictEqual([1, 2, 3]);
  expect(() => query(p, 'f.iterator()')).toThrow(
    '.iterator is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.arrowFnReturningIterator = x', 'x', true)).toStrictEqual(
    f.arrowFnReturningIterator
  );
  expect(qvar(p, 'f.arrowFnReturningIterator() = x', 'x')).toStrictEqual([
    1,
    2,
    3,
  ]);

  expect(qvar(p, 'f.array = x', 'x')).toStrictEqual([f.array]);
  expect(() => query(p, 'f.array()')).toThrow(
    '.array is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.object = x', 'x')).toStrictEqual([f.object]);
  expect(() => query(p, 'f.object()')).toThrow(
    '.object is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.validCustomIterator = x', 'x')).toStrictEqual([1, 2, 3]);
  expect(() => query(p, 'f.validCustomIterator()')).toThrow(
    '.validCustomIterator is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.invalidCustomIterator = x', 'x', true)).toStrictEqual(
    f.invalidCustomIterator
  );
  expect(() => query(p, 'f.invalidCustomIterator()')).toThrow(
    '.invalidCustomIterator is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.map = x', 'x', true)).toStrictEqual(f.map);
  expect(() => query(p, 'f.map()')).toThrow(
    '.map is not a function at line 1, column 1'
  );

  expect(qvar(p, 'f.set = x', 'x', true)).toStrictEqual(f.set);
  expect(() => query(p, 'f.set()')).toThrow(
    '.set is not a function at line 1, column 1'
  );
});
