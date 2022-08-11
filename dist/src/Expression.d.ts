import type { PolarOperator } from './types';
/** Polar expression. */
export declare class Expression {
    readonly operator: PolarOperator;
    readonly args: unknown[];
    constructor(operator: PolarOperator, args: unknown[]);
}
