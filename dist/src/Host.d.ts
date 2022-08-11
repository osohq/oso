import type { Polar as FfiPolar } from './polar_wasm_api';
import type { Class, ClassParams, HostOpts, PolarComparisonOperator, PolarTerm, HostTypes, obj } from './types';
import { UserType } from './types';
import { Adapter } from './filter';
/**
 * Translator between Polar and JavaScript.
 *
 * @internal
 */
export declare class Host<Query, Resource> {
    #private;
    types: HostTypes;
    adapter: Adapter<Query, Resource>;
    /**
     * Shallow clone a host to extend its state for the duration of a particular
     * query without modifying the longer-lived [[`Polar`]] host state.
     *
     * @internal
     */
    static clone<Query, Resource>(host: Host<Query, Resource>, opts: Partial<HostOpts>): Host<Query, Resource>;
    /** @internal */
    constructor(ffiPolar: FfiPolar, opts: HostOpts);
    /**
     * Fetch a JavaScript class from the class cache.
     *
     * @param name Class name to look up.
     *
     * @internal
     */
    private getClass;
    /**
     * Get user type for `cls`.
     *
     * @param cls Class or class name.
     */
    getType<Type extends Class>(cls?: Type | string): UserType<Type> | undefined;
    /**
     * Return user types that are registered with Host.
     */
    private distinctUserTypes;
    serializeTypes(): obj;
    /**
     * Store a JavaScript class in the class cache.
     *
     * @param cls Class to cache.
     * @param params Optional parameters.
     *
     * @internal
     */
    cacheClass(cls: Class, params?: ClassParams): string;
    /**
     * Return cached instances.
     *
     * Only used by the test suite.
     *
     * @internal
     */
    instances(): unknown[];
    /**
     * Check if an instance exists in the instance cache.
     *
     * @internal
     */
    hasInstance(id: number): boolean;
    /**
     * Fetch a JavaScript instance from the instance cache.
     *
     * Public for the test suite.
     *
     * @internal
     */
    getInstance(id: number): unknown;
    /**
     * Store a JavaScript instance in the instance cache, fetching a new instance
     * ID from the Polar VM if an ID is not provided.
     *
     * @internal
     */
    cacheInstance(instance: unknown, id?: number): number;
    /**
     * Register the MROs of all registered classes.
     */
    registerMros(): void;
    /**
     * Construct a JavaScript instance and store it in the instance cache.
     *
     * @internal
     */
    makeInstance(name: string, fields: PolarTerm[], id: number): Promise<void>;
    /**
     * Check if the left class is more specific than the right class with respect
     * to the given instance.
     *
     * @internal
     */
    isSubspecializer(id: number, left: string, right: string): Promise<boolean>;
    /**
     * Check if the left class is a subclass of the right class.
     *
     * @internal
     */
    isSubclass(left: string, right: string): boolean;
    /**
     * Check if the given instance is an instance of a particular class.
     *
     * @internal
     */
    isa(polarInstance: PolarTerm, name: string): Promise<boolean>;
    /**
     * Check if a sequence of field accesses on the given class is an
     * instance of another class.
     *
     * @internal
     */
    isaWithPath(baseTag: string, path: PolarTerm[], classTag: string): Promise<boolean>;
    /**
     * Check if the given instances conform to the operator.
     *
     * @internal
     */
    externalOp(op: PolarComparisonOperator, leftTerm: PolarTerm, rightTerm: PolarTerm): Promise<boolean>;
    /**
     * Turn a JavaScript value into a Polar term that's ready to be sent to the
     * Polar VM.
     *
     * @internal
     */
    toPolar(v: unknown): PolarTerm;
    /**
     * Turn a Polar term from the Polar VM into a JavaScript value.
     *
     * @internal
     */
    toJs(v: PolarTerm): Promise<unknown>;
}
