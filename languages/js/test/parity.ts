import path from 'path';
import { Oso } from '../src/Oso';

const oso = new Oso();

class A {
  readonly x: string;

  constructor(x: string) {
    this.x = x;
  }

  foo() {
    return -1;
  }
}
oso.registerClass(A);

class D extends A {}

namespace B {
  export class C {
    readonly y: string;

    constructor(y: string) {
      this.y = y;
    }

    foo() {
      return -1;
    }
  }
}

oso.registerClass(B.C, 'C');

class E extends Array<number> {
  constructor(array: number[]) {
    super(...array);

    Object.setPrototypeOf(this, E.prototype);
  }

  static plus_one(x: number) {
    return x + 1;
  }
}
oso.registerClass(E)

// This path has the same nesting for development and the parity test jobs by sheer coincidence.
// In tests it's `languages/js/test/parity.ts`
// In parity tests it's `js_package/dist/test/parity.js`
// In both these cases the relative path to the test.polar file is the same.
oso.loadFile(path.resolve(__dirname, '../../../test/test.polar'));

if (!oso.isAllowed('a', 'b', 'c')) throw new Error();

// Test that a built in string method can be called.
oso.loadStr('?= x = "hello world!" and x.endsWith("world!");');

// Test that a custom error type is thrown.
let exceptionThrown = false;
try {
  oso.loadStr('missingSemicolon()');
} catch (e) {
  const expectedName = 'ParseError::UnrecognizedEOF';
  const expectedMessage =
    'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19';
  if (e.name === expectedName && e.message === expectedMessage)
    exceptionThrown = true;
} finally {
  if (!exceptionThrown) throw new Error();
}

if (
  [
    oso.queryRule('specializers', new D('hello'), new B.C('hello')).next().done,
    oso.queryRule('floatLists').next().done,
    oso.queryRule('intDicts').next().done,
    oso.queryRule('comparisons').next().done,
    oso.queryRule('testForall').next().done,
    oso.queryRule('testRest').next().done,
    oso.queryRule('testMatches', new A('hello')).next().done,
    oso.queryRule('testMethodCalls', new A('hello'), new B.C('hello')).next()
      .done,
    oso.queryRule('testOr').next().done,
    // oso.queryRule('testHttpAndPathMapper').next().done,
  ].some(v => v)
)
  throw new Error();

// Test that cut doesn't return anything.
if (!oso.queryRule('testCut').next().done) throw new Error();

// Test that a constant can be called.
oso.registerConstant('Math', Math);
oso.loadStr('?= Math.acos(1.0) = 0;');

// Test built-in type specializers.
if (
  [
    oso.query('builtinSpecializers(true, "Boolean")').next().done,
    !oso.query('builtinSpecializers(false, "Boolean")').next().done,
    oso.query('builtinSpecializers(2, "Integer")').next().done,
    oso.query('builtinSpecializers(1, "Integer")').next().done,
    !oso.query('builtinSpecializers(0, "Integer")').next().done,
    !oso.query('builtinSpecializers(-1, "Integer")').next().done,
    oso.query('builtinSpecializers(1.0, "Float")').next().done,
    !oso.query('builtinSpecializers(0.0, "Float")').next().done,
    !oso.query('builtinSpecializers(-1.0, "Float")').next().done,
    oso.query('builtinSpecializers(["foo", "bar", "baz"], "List")').next().done,
    !oso.query('builtinSpecializers(["bar", "foo", "baz"], "List")').next()
      .done,
    oso.query('builtinSpecializers({foo: "foo"}, "Dictionary")').next().done,
    !oso.query('builtinSpecializers({foo: "bar"}, "Dictionary")').next().done,
    oso.query('builtinSpecializers("foo", "String")').next().done,
    !oso.query('builtinSpecializers("bar", "String")').next().done,
    // oso.query('testFunctions()').next().done,
  ].some(v => v)
)
  throw new Error();

console.log('tests pass');
