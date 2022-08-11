"use strict";
var _a, _b;
Object.defineProperty(exports, "__esModule", { value: true });
exports.isString = exports.defaultEqualityFn = exports.isObj = exports.isConstructor = exports.printError = exports.PROMPT = exports.readFile = exports.parseQueryEvent = exports.repr = exports.ancestors = void 0;
const util_1 = require("util");
const fs_1 = require("fs");
const errors_1 = require("./errors");
const types_1 = require("./types");
const isEqual = require("lodash.isequal");
/**
 * Assemble the prototypal inheritance chain of a class.
 *
 * @returns The inheritance chain as a list of prototypes in most-to-least
 * specific order.
 *
 * @internal
 */
function ancestors(cls) {
    if (!isConstructor(cls))
        return [];
    const ancestors = [cls];
    function next(current) {
        const parent = Object.getPrototypeOf(current); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
        if (parent === Function.prototype)
            return;
        if (!isConstructor(parent))
            return;
        ancestors.push(parent);
        next(parent);
    }
    next(cls);
    return ancestors;
}
exports.ancestors = ancestors;
/**
 * Stringify a value.
 *
 * @returns A string representation of the input value.
 *
 * @internal
 */
function repr(x) {
    return util_1.inspect(x);
}
exports.repr = repr;
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary
 * into a valid [[`QueryEvent`]].
 *
 * @internal
 */
function parseQueryEvent(event) {
    try {
        if (exports.isString(event))
            throw new Error();
        switch (true) {
            case event['Done'] !== undefined:
                return { kind: types_1.QueryEventKind.Done };
            case event['Result'] !== undefined:
                return parseResult(event['Result']);
            case event['MakeExternal'] !== undefined:
                return parseMakeExternal(event['MakeExternal']);
            case event['NextExternal'] !== undefined:
                return parseNextExternal(event['NextExternal']);
            case event['ExternalCall'] !== undefined:
                return parseExternalCall(event['ExternalCall']);
            case event['ExternalIsSubSpecializer'] !== undefined:
                return parseExternalIsSubspecializer(event['ExternalIsSubSpecializer']);
            case event['ExternalIsSubclass'] !== undefined:
                return parseExternalIsSubclass(event['ExternalIsSubclass']);
            case event['ExternalIsa'] !== undefined:
                return parseExternalIsa(event['ExternalIsa']);
            case event['ExternalIsaWithPath'] !== undefined:
                return parseExternalIsaWithPath(event['ExternalIsaWithPath']);
            case event['Debug'] !== undefined:
                return parseDebug(event['Debug']);
            case event['ExternalOp'] !== undefined:
                return parseExternalOp(event['ExternalOp']);
            default:
                throw new Error();
        }
    }
    catch (e) {
        if (e instanceof errors_1.PolarError)
            throw e;
        throw new errors_1.InvalidQueryEventError(JSON.stringify(event));
    }
}
exports.parseQueryEvent = parseQueryEvent;
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`Result`]].
 *
 * @internal
 */
function parseResult(event) {
    if (!isObj(event))
        throw new Error();
    const { bindings } = event;
    if (!isMapOfPolarTerms(bindings))
        throw new Error();
    return { kind: types_1.QueryEventKind.Result, data: { bindings } };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`MakeExternal`]].
 *
 * @internal
 */
function parseMakeExternal(event) {
    if (!isObj(event))
        throw new Error();
    const { instance_id: instanceId } = event;
    if (!isSafeInteger(instanceId))
        throw new Error();
    const ctor = event['constructor'];
    if (!types_1.isPolarTerm(ctor))
        throw new Error();
    if (!types_1.isPolarPredicate(ctor.value))
        throw new Error();
    // TODO(gj): can we remove this kwargs check?
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (ctor.value.Call.kwargs)
        throw new errors_1.KwargsError();
    const { name: tag, args: fields } = ctor.value.Call;
    if (!exports.isString(tag))
        throw new Error();
    if (!isArrayOf(fields, types_1.isPolarTerm))
        throw new Error();
    return {
        kind: types_1.QueryEventKind.MakeExternal,
        data: { fields, instanceId, tag },
    };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`NextExternal`]].
 *
 * @internal
 */
function parseNextExternal(event) {
    if (!isObj(event))
        throw new Error();
    const { call_id: callId, iterable } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!types_1.isPolarTerm(iterable))
        throw new Error();
    return { kind: types_1.QueryEventKind.NextExternal, data: { callId, iterable } };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalCall`]].
 *
 * @internal
 */
function parseExternalCall(event) {
    if (!isObj(event))
        throw new Error();
    const { args, kwargs, attribute, call_id: callId, instance } = event;
    if (args !== undefined && !isArrayOf(args, types_1.isPolarTerm))
        throw new Error();
    if (kwargs)
        throw new errors_1.KwargsError();
    if (!exports.isString(attribute))
        throw new Error();
    if (!isSafeInteger(callId))
        throw new Error();
    if (!types_1.isPolarTerm(instance))
        throw new Error();
    return {
        kind: types_1.QueryEventKind.ExternalCall,
        data: { args, attribute, callId, instance },
    };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsSubspecializer`]].
 *
 * @internal
 */
function parseExternalIsSubspecializer(event) {
    if (!isObj(event))
        throw new Error();
    const { call_id: callId, instance_id: instanceId, left_class_tag: leftTag, right_class_tag: rightTag, } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!isSafeInteger(instanceId))
        throw new Error();
    if (!exports.isString(leftTag))
        throw new Error();
    if (!exports.isString(rightTag))
        throw new Error();
    return {
        kind: types_1.QueryEventKind.ExternalIsSubspecializer,
        data: { callId, instanceId, leftTag, rightTag },
    };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsSubclass`]].
 *
 * @internal
 */
function parseExternalIsSubclass(event) {
    if (!isObj(event))
        throw new Error();
    const { call_id: callId, left_class_tag: leftTag, right_class_tag: rightTag, } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!exports.isString(leftTag))
        throw new Error();
    if (!exports.isString(rightTag))
        throw new Error();
    return {
        kind: types_1.QueryEventKind.ExternalIsSubclass,
        data: { callId, leftTag, rightTag },
    };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsa`]].
 *
 * @internal
 */
function parseExternalIsa(event) {
    if (!isObj(event))
        throw new Error();
    const { call_id: callId, instance, class_tag: tag } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!types_1.isPolarTerm(instance))
        throw new Error();
    if (!exports.isString(tag))
        throw new Error();
    return { kind: types_1.QueryEventKind.ExternalIsa, data: { callId, instance, tag } };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsa`]].
 *
 * @internal
 */
function parseExternalIsaWithPath(event) {
    if (!isObj(event))
        throw new Error();
    const { path, call_id: callId, base_tag: baseTag, class_tag: classTag, } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!exports.isString(baseTag))
        throw new Error();
    if (!exports.isString(classTag))
        throw new Error();
    if (!isArrayOf(path, types_1.isPolarTerm))
        throw new Error();
    return {
        kind: types_1.QueryEventKind.ExternalIsaWithPath,
        data: { callId, baseTag, path, classTag },
    };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalOp`]].
 *
 * @internal
 */
function parseExternalOp(event) {
    if (!isObj(event))
        throw new Error();
    const { call_id: callId, args, operator } = event;
    if (!isSafeInteger(callId))
        throw new Error();
    if (!isArrayOf(args, types_1.isPolarTerm) || args.length !== 2)
        throw new Error();
    if (!exports.isString(operator))
        throw new Error();
    if (!types_1.isPolarComparisonOperator(operator))
        throw new errors_1.PolarError(`Unsupported external operation '${repr(args[0])} ${operator} ${repr(args[1])}'`);
    return { kind: types_1.QueryEventKind.ExternalOp, data: { args, callId, operator } };
}
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`Debug`]].
 *
 * @internal
 */
function parseDebug(event) {
    if (!isObj(event))
        throw new Error();
    const { message } = event;
    if (!exports.isString(message))
        throw new Error();
    return { kind: types_1.QueryEventKind.Debug, data: { message } };
}
/**
 * Promisified version of the pre-`fs/promises` asynchronous `fs.readFile`
 * function since none of the following work on all Node.js versions we want to
 * support (>= 10):
 *
 * ```ts
 * import { readFile } from 'fs/promises';
 * import { promises } from 'fs';
 * const { readFile } = require('fs/promises');
 * ```
 *
 * @internal
 */
function readFile(file) {
    return new Promise((res, rej) => fs_1.readFile(file, { encoding: 'utf8' }, (err, contents) => err === null ? res(contents) : rej(err)));
}
exports.readFile = readFile;
// Optional ANSI escape sequences for the REPL.
let RESET = '';
let FG_BLUE = '';
let FG_RED = '';
if (typeof ((_a = process === null || process === void 0 ? void 0 : process.stdout) === null || _a === void 0 ? void 0 : _a.getColorDepth) === 'function' &&
    process.stdout.getColorDepth() >= 4 &&
    typeof ((_b = process === null || process === void 0 ? void 0 : process.stderr) === null || _b === void 0 ? void 0 : _b.getColorDepth) === 'function' &&
    process.stderr.getColorDepth() >= 4) {
    RESET = '\x1b[0m';
    FG_BLUE = '\x1b[34m';
    FG_RED = '\x1b[31m';
}
/** @internal */
exports.PROMPT = FG_BLUE + 'query> ' + RESET;
/** @internal */
function printError(e) {
    console.error(FG_RED + e.name + RESET);
    console.error(e.message);
}
exports.printError = printError;
/**
 * https://stackoverflow.com/a/46759625
 *
 * @internal
 */
function isConstructor(f) {
    try {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        Reflect.construct(String, [], f);
        return true;
    }
    catch (e) {
        return false;
    }
}
exports.isConstructor = isConstructor;
/**
 * Type guard to test if a value is a [[`obj`]].
 *
 * @internal
 */
function isObj(x) {
    return typeof x === 'object' && x !== null;
}
exports.isObj = isObj;
/**
 * Default equality function used by Oso
 */
function defaultEqualityFn(a, b) {
    return isEqual(a, b);
}
exports.defaultEqualityFn = defaultEqualityFn;
/**
 * Type guard to test if `x` is a `string`.
 *
 * @internal
 */
const isString = (x) => typeof x === 'string';
exports.isString = isString;
/**
 * Type guard to test if a value is an ES6 Map with string keys and PolarTerm
 * values.
 *
 * @internal
 */
const isMapOfPolarTerms = (x) => x instanceof Map &&
    [...x.keys()].every(exports.isString) &&
    [...x.values()].every(types_1.isPolarTerm);
const isArrayOf = (x, p) => Array.isArray(x) && x.every(p);
/**
 * Type guard to test if a value is a safe integer.
 *
 * @internal
 */
function isSafeInteger(x) {
    return Number.isSafeInteger(x);
}
//# sourceMappingURL=helpers.js.map