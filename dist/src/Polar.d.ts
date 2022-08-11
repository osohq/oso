import { Host } from './Host';
import { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import type { Class, ClassParams, Options, QueryOpts, QueryResult } from './types';
/** Create and manage an instance of the Polar runtime. */
export declare class Polar<Query, Resource> {
    #private;
    constructor(opts?: Options);
    /**
     * Free the underlying WASM instance.
     *
     * Invariant: ensure that you do *not* do anything else with an instance
     * after calling `free()` on it.
     *
     * This should *not* be something you need to do during the course of regular
     * usage. It's generally only useful for scenarios where large numbers of
     * instances are spun up and not cleanly reaped by the GC, such as during a
     * long-running test process in 'watch' mode.
     */
    free(): void;
    /**
     * Process messages received from the Polar VM.
     *
     * @internal
     */
    private processMessages;
    /**
     * Clear rules from the Polar KB, but
     * retain all registered classes and constants.
     */
    clearRules(): void;
    /**
     * Load Polar policy files.
     */
    loadFiles(filenames: string[]): Promise<void>;
    /**
     * Load a Polar policy file.
     *
     * @deprecated `Oso.loadFile` has been deprecated in favor of `Oso.loadFiles`
     * as of the 0.20 release. Please see changelog for migration instructions:
     * https://docs.osohq.com/project/changelogs/2021-09-15.html
     */
    loadFile(filename: string): Promise<void>;
    /**
     * Load a Polar policy string.
     */
    loadStr(contents: string, filename?: string): Promise<void>;
    private loadSources;
    private checkInlineQueries;
    /**
     * Query for a Polar predicate or string.
     */
    query(q: Predicate | string, opts?: QueryOpts): QueryResult;
    /**
     * Query for a Polar rule.
     */
    queryRule(opts: QueryOpts, name: string, ...args: unknown[]): QueryResult;
    queryRule(name: string, ...args: unknown[]): QueryResult;
    /**
     * Query for a Polar rule, returning true if there are any results.
     */
    queryRuleOnce(name: string, ...args: unknown[]): Promise<boolean>;
    /**
     * Register a JavaScript class for use in Polar policies.
     *
     * @param cls The class to register.
     * @param params An optional object with extra parameters.
     */
    registerClass(cls: Class, params?: ClassParams): void;
    /**
     * Register a JavaScript value for use in Polar policies.
     */
    registerConstant(value: unknown, name: string): void;
    getHost(): Host<Query, Resource>;
    getFfi(): FfiPolar;
    /** Start a REPL session. */
    repl(files?: string[]): Promise<void>;
    /**
     * Evaluate REPL input.
     *
     * @internal
     */
    private evalReplInput;
}
