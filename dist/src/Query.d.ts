import type { Query as FfiQuery } from './polar_wasm_api';
import { Host } from './Host';
import type { QueryResult } from './types';
/**
 * A single Polar query.
 *
 * @internal
 */
export declare class Query<AdapterQuery, Resource> {
    #private;
    results: QueryResult;
    constructor(ffiQuery: FfiQuery, host: Host<AdapterQuery, Resource>, bindings?: Map<string, unknown>);
    /**
     * Process messages received from the Polar VM.
     *
     * @internal
     */
    private bind;
    /**
     * Process messages received from the Polar VM.
     *
     * @internal
     */
    private processMessages;
    /**
     * Send result of predicate check back to the Polar VM.
     *
     * @internal
     */
    private questionResult;
    /**
     * Send next result of JavaScript method call or property lookup to the Polar
     * VM.
     *
     * @internal
     */
    private callResult;
    /**
     * Retrieve the next result from a registered call and prepare it for
     * transmission back to the Polar VM.
     *
     * @internal
     */
    private nextCallResult;
    /**
     * Send application error back to the Polar VM.
     *
     * @internal
     */
    private applicationError;
    /**
     * Handle an external call on a relation.
     *
     * @internal
     */
    private handleRelation;
    /**
     * Handle an application call.
     *
     * @internal
     */
    private handleCall;
    private handleNextExternal;
    /**
     * Create an `AsyncGenerator` that can be polled to advance the query loop.
     *
     * @internal
     */
    private start;
}
