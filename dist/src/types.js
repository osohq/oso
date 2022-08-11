"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UserType = exports.Dict = exports.isAsyncIterable = exports.isIterable = exports.isIterableIterator = exports.QueryEventKind = exports.isPolarTerm = exports.isPolarPattern = exports.isPolarExpression = exports.isPolarInstance = exports.isPolarVariable = exports.isPolarPredicate = exports.isPolarDict = exports.isPolarList = exports.isPolarBool = exports.isPolarNum = exports.isPolarComparisonOperator = exports.isPolarStr = void 0;
const helpers_1 = require("./helpers");
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar string.
 *
 * @internal
 */
function isPolarStr(v) {
    return v.String !== undefined;
}
exports.isPolarStr = isPolarStr;
/**
 * Type guard to test if a string received from across the WebAssembly
 * boundary is a PolarComparisonOperator.
 *
 * @internal
 */
function isPolarComparisonOperator(s) {
    return s in comparisonOperators;
}
exports.isPolarComparisonOperator = isPolarComparisonOperator;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar numeric.
 *
 * @internal
 */
function isPolarNum(v) {
    return v.Number !== undefined;
}
exports.isPolarNum = isPolarNum;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar boolean.
 *
 * @internal
 */
function isPolarBool(v) {
    return v.Boolean !== undefined;
}
exports.isPolarBool = isPolarBool;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar list.
 *
 * @internal
 */
function isPolarList(v) {
    return v.List !== undefined;
}
exports.isPolarList = isPolarList;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar dictionary.
 *
 * @internal
 */
function isPolarDict(v) {
    return v.Dictionary !== undefined;
}
exports.isPolarDict = isPolarDict;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar predicate.
 *
 * @internal
 */
function isPolarPredicate(v) {
    return v.Call !== undefined;
}
exports.isPolarPredicate = isPolarPredicate;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar variable.
 *
 * @internal
 */
function isPolarVariable(v) {
    return v.Variable !== undefined;
}
exports.isPolarVariable = isPolarVariable;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar application instance.
 *
 * @internal
 */
function isPolarInstance(v) {
    return v.ExternalInstance !== undefined;
}
exports.isPolarInstance = isPolarInstance;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar expression.
 *
 * @internal
 */
function isPolarExpression(v) {
    return v.Expression !== undefined;
}
exports.isPolarExpression = isPolarExpression;
/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar pattern.
 *
 * @internal
 */
function isPolarPattern(v) {
    return v.Pattern !== undefined;
}
exports.isPolarPattern = isPolarPattern;
/**
 * Type guard to test if a JSON payload received from across the WebAssembly
 * boundary contains a valid Polar value.
 *
 * @internal
 */
function isPolarValue(x) {
    if (!helpers_1.isObj(x))
        return false;
    const v = x;
    return (isPolarStr(v) ||
        isPolarNum(v) ||
        isPolarBool(v) ||
        isPolarList(v) ||
        isPolarDict(v) ||
        isPolarPredicate(v) ||
        isPolarVariable(v) ||
        isPolarInstance(v) ||
        isPolarExpression(v) ||
        isPolarPattern(v));
}
/**
 * Type guard to test if a JSON payload received from across the WebAssembly
 * boundary contains a valid Polar term.
 *
 * @internal
 */
function isPolarTerm(v) {
    if (!helpers_1.isObj(v))
        return false;
    return isPolarValue(v.value);
}
exports.isPolarTerm = isPolarTerm;
/**
 * Polar comparison operators.
 *
 * Currently, these are the only operators supported for external operations.
 *
 * @internal
 */
const comparisonOperators = {
    Eq: 'Eq',
    Geq: 'Geq',
    Gt: 'Gt',
    Leq: 'Leq',
    Lt: 'Lt',
    Neq: 'Neq',
};
/**
 * Polar operators.
 *
 * @internal
 */
const operators = {
    Add: 'Add',
    And: 'And',
    Assign: 'Assign',
    Cut: 'Cut',
    Debug: 'Debug',
    Div: 'Div',
    Dot: 'Dot',
    ForAll: 'ForAll',
    In: 'In',
    Isa: 'Isa',
    Mod: 'Mod',
    Mul: 'Mul',
    New: 'New',
    Not: 'Not',
    Or: 'Or',
    Print: 'Print',
    Rem: 'Rem',
    Sub: 'Sub',
    Unify: 'Unify',
    ...comparisonOperators,
};
/**
 * Union of all [[`QueryEvent`]] types.
 *
 * @internal
 */
var QueryEventKind;
(function (QueryEventKind) {
    QueryEventKind[QueryEventKind["Debug"] = 0] = "Debug";
    QueryEventKind[QueryEventKind["Done"] = 1] = "Done";
    QueryEventKind[QueryEventKind["ExternalCall"] = 2] = "ExternalCall";
    QueryEventKind[QueryEventKind["ExternalIsa"] = 3] = "ExternalIsa";
    QueryEventKind[QueryEventKind["ExternalIsaWithPath"] = 4] = "ExternalIsaWithPath";
    QueryEventKind[QueryEventKind["ExternalIsSubspecializer"] = 5] = "ExternalIsSubspecializer";
    QueryEventKind[QueryEventKind["ExternalIsSubclass"] = 6] = "ExternalIsSubclass";
    QueryEventKind[QueryEventKind["ExternalOp"] = 7] = "ExternalOp";
    QueryEventKind[QueryEventKind["MakeExternal"] = 8] = "MakeExternal";
    QueryEventKind[QueryEventKind["NextExternal"] = 9] = "NextExternal";
    QueryEventKind[QueryEventKind["Result"] = 10] = "Result";
})(QueryEventKind = exports.QueryEventKind || (exports.QueryEventKind = {}));
/**
 * Type guard to test if a value conforms to both the iterable and iterator
 * protocols. This is basically a slightly relaxed check for whether the value
 * is a `Generator`.
 *
 * @internal
 */
function isIterableIterator(x) {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    return typeof (x === null || x === void 0 ? void 0 : x.next) === 'function' && isIterable(x);
}
exports.isIterableIterator = isIterableIterator;
/**
 * Type guard to test if a value is an `Iterable`.
 *
 * @internal
 */
function isIterable(x) {
    try {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        return Symbol.iterator in x;
    }
    catch (e) {
        if (e instanceof TypeError)
            return false;
        throw e;
    }
}
exports.isIterable = isIterable;
/**
 * Type guard to test if a value is an `AsyncIterable`.
 *
 * @internal
 */
function isAsyncIterable(x) {
    try {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        return Symbol.asyncIterator in x;
    }
    catch (e) {
        if (e instanceof TypeError)
            return false;
        throw e;
    }
}
exports.isAsyncIterable = isAsyncIterable;
/**
 * JS analogue of Polar's Dictionary type.
 *
 * Polar dictionaries allow field access via the dot operator, which mirrors
 * the way JS objects behave. However, if we translate Polar dictionaries into
 * JS objects, we lose the ability to distinguish between dictionaries and
 * instances, since all JS instances are objects. By subclassing `Object`, we
 * can use `instanceof` to determine if a JS value should be serialized as a
 * Polar dictionary or external instance.
 *
 * @internal
 */
class Dict extends Object {
}
exports.Dict = Dict;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
class UserType {
    constructor({ name, cls, id, fields, isaCheck }) {
        this.name = name;
        this.cls = cls;
        this.fields = fields;
        this.id = id;
        this.isaCheck = isaCheck;
    }
}
exports.UserType = UserType;
//# sourceMappingURL=types.js.map