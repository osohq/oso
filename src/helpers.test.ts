import { ancestors } from './helpers';

describe('ancestors', () => {
  test('works with user-defined classes', () => {
    class A {}
    class B extends A {}
    expect(ancestors(A)).toStrictEqual([A]);
    expect(ancestors(B)).toStrictEqual([B, A]);
  });

  test('works with user-defined classes that inherit from built-in classes', () => {
    class A extends String {}
    class B extends A {}
    expect(ancestors(A)).toStrictEqual([A, String]);
    expect(ancestors(B)).toStrictEqual([B, A, String]);
  });

  test('works with built-in classes', () => {
    expect(ancestors(Object)).toStrictEqual([Object]);
    expect(ancestors(String)).toStrictEqual([String]);
  });

  test('non-classes have empty MROs', () => {
    expect(ancestors({})).toStrictEqual([]);
    expect(ancestors({ a: 1 })).toStrictEqual([]);
    expect(ancestors(null)).toStrictEqual([]);
    expect(ancestors(undefined)).toStrictEqual([]);
  });
});
