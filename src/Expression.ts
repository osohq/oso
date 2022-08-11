import type { PolarOperator } from './types';

/** Polar expression. */
export class Expression {
  readonly operator: PolarOperator;
  readonly args: unknown[];

  constructor(operator: PolarOperator, args: unknown[]) {
    this.operator = operator;
    this.args = args;
  }
}
