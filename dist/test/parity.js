"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const Oso_1 = require("../src/Oso");
const Variable_1 = require("../src/Variable");
const path_1 = require("path");
const oso = new Oso_1.Oso();
class A {
    constructor(x) {
        this.x = x;
    }
    foo() {
        return -1;
    }
}
oso.registerClass(A);
class D extends A {
}
// eslint-disable-next-line @typescript-eslint/no-namespace
var B;
(function (B) {
    class C {
        constructor(y) {
            this.y = y;
        }
        foo() {
            return -1;
        }
    }
    B.C = C;
})(B || (B = {}));
oso.registerClass(B.C, { name: 'C' });
class E {
    static sum(args) {
        return args.reduce((a, b) => {
            return a + b;
        }, 0);
    }
}
oso.registerClass(E);
void (async function () {
    // Test that a custom error type is thrown.
    let exceptionThrown = false;
    try {
        await oso.loadStr('missingSemicolon()');
    }
    catch (e) {
        const expectedName = 'ParseError::UnrecognizedEOF';
        const expectedMessage = 'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19';
        const { name, message } = e;
        if (name === expectedName && message.startsWith(expectedMessage))
            exceptionThrown = true;
    }
    finally {
        if (!exceptionThrown)
            throw new Error(); // eslint-disable-line no-unsafe-finally
    }
    // Test that a built in string method can be called.
    await oso.loadStr('?= x = "hello world!" and x.endsWith("world!");');
    oso.clearRules();
    // Test that a constant can be called.
    oso.registerConstant(Math, 'MyMath');
    await oso.loadStr('?= MyMath.acos(1.0) = 0;');
    oso.clearRules();
    // Test deref behaviour
    await oso.loadStr('?= x = 1 and E.sum([x, 2, x]) = 4 and [3, 2, x].indexOf(1) = 2;');
    oso.clearRules();
    // This path has the same nesting for development and the parity test jobs by sheer coincidence.
    // In tests it's `languages/js/test/parity.ts`
    // In parity tests it's `js_package/dist/test/parity.js`
    // In both these cases the relative path to the test.polar file is the same.
    await oso.loadFiles([path_1.join(__dirname, '../../../test/test.polar')]);
    if (!(await oso.isAllowed('a', 'b', 'c')))
        throw new Error();
    if ([
        (await oso
            .queryRule('specializers', new D('hello'), new B.C('hello'))
            .next()).done,
        (await oso.queryRule('floatLists').next()).done,
        (await oso.queryRule('intDicts').next()).done,
        (await oso.queryRule('comparisons').next()).done,
        (await oso.queryRule('testForall').next()).done,
        (await oso.queryRule('testRest').next()).done,
        (await oso.queryRule('testMatches', new A('hello')).next()).done,
        (await oso
            .queryRule('testMethodCalls', new A('hello'), new B.C('hello'))
            .next()).done,
        (await oso.queryRule('testOr').next()).done,
        (await oso.queryRule('testUnifyClass', A).next()).done,
    ].some(v => v))
        throw new Error();
    // Test that cut doesn't return anything.
    if (!(await oso.queryRule('testCut').next()).done)
        throw new Error();
    // test iterables work
    // if ((await oso.queryRule('testIterables').next()).done) throw new Error();
    // Test built-in type specializers.
    if ([
        (await oso.query('builtinSpecializers(true, "Boolean")').next()).done,
        !(await oso.query('builtinSpecializers(false, "Boolean")').next()).done,
        (await oso.query('builtinSpecializers(2, "Integer")').next()).done,
        (await oso.query('builtinSpecializers(1, "Integer")').next()).done,
        !(await oso.query('builtinSpecializers(0, "Integer")').next()).done,
        !(await oso.query('builtinSpecializers(-1, "Integer")').next()).done,
        (await oso.query('builtinSpecializers(1.0, "Float")').next()).done,
        !(await oso.query('builtinSpecializers(0.0, "Float")').next()).done,
        !(await oso.query('builtinSpecializers(-1.0, "Float")').next()).done,
        (await oso
            .query('builtinSpecializers(["foo", "bar", "baz"], "List")')
            .next()).done,
        !(await oso
            .query('builtinSpecializers(["bar", "foo", "baz"], "List")')
            .next()).done,
        (await oso
            .query('builtinSpecializers({foo: "foo"}, "Dictionary")')
            .next()).done,
        !(await oso
            .query('builtinSpecializers({foo: "bar"}, "Dictionary")')
            .next()).done,
        (await oso.query('builtinSpecializers("foo", "String")').next()).done,
        !(await oso.query('builtinSpecializers("bar", "String")').next()).done,
        !(await oso.query('builtinSpecializers(1, "IntegerWithFields")').next())
            .done,
        !(await oso
            .query('builtinSpecializers(2, "IntegerWithGarbageFields")')
            .next()).done,
        !(await oso
            .query('builtinSpecializers({}, "DictionaryWithFields")')
            .next()).done,
        !(await oso
            .query('builtinSpecializers({z: 1}, "DictionaryWithFields")')
            .next()).done,
        (await oso
            .query('builtinSpecializers({y: 1}, "DictionaryWithFields")')
            .next()).done,
    ].some(v => v))
        throw new Error();
    // Test unspecialized rule ordering
    const result = oso.queryRule('testUnspecializedRuleOrder', 'foo', 'bar', new Variable_1.Variable('z'));
    if ((await result.next()).value.get('z') !== 1)
        throw new Error();
    if ((await result.next()).value.get('z') !== 2)
        throw new Error();
    if ((await result.next()).value.get('z') !== 3)
        throw new Error();
    // Test that a custom error type is thrown.
    exceptionThrown = false;
    try {
        await oso.query('testUnhandledPartial()').next();
    }
    catch (e) {
        const expectedName = 'RuntimeError::UnhandledPartial';
        const { name, message } = e;
        if (name === expectedName &&
            message.startsWith('Found an unhandled partial'))
            exceptionThrown = true;
    }
    finally {
        if (!exceptionThrown)
            throw new Error(); // eslint-disable-line no-unsafe-finally
    }
    console.log('tests pass');
})();
//# sourceMappingURL=parity.js.map