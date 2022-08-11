import type { Class, obj, QueryEvent } from './types';
/**
 * Assemble the prototypal inheritance chain of a class.
 *
 * @returns The inheritance chain as a list of prototypes in most-to-least
 * specific order.
 *
 * @internal
 */
export declare function ancestors(cls: unknown): unknown[];
/**
 * Stringify a value.
 *
 * @returns A string representation of the input value.
 *
 * @internal
 */
export declare function repr(x: unknown): string;
/**
 * Try to parse a JSON payload received from across the WebAssembly boundary
 * into a valid [[`QueryEvent`]].
 *
 * @internal
 */
export declare function parseQueryEvent(event: string | obj): QueryEvent;
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
export declare function readFile(file: string): Promise<string>;
/** @internal */
export declare const PROMPT: string;
/** @internal */
export declare function printError(e: Error): void;
/**
 * https://stackoverflow.com/a/46759625
 *
 * @internal
 */
export declare function isConstructor(f: unknown): f is Class;
/**
 * Type guard to test if a value is a [[`obj`]].
 *
 * @internal
 */
export declare function isObj(x: unknown): x is obj;
/**
 * Default equality function used by Oso
 */
export declare function defaultEqualityFn(a: unknown, b: unknown): boolean;
/**
 * Type guard to test if `x` is a `string`.
 *
 * @internal
 */
export declare const isString: (x: unknown) => x is string;
