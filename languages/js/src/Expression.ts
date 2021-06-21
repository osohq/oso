import type { PolarOperator } from './types';

/** Polar expression. */
export class Expression {
  readonly operator: PolarOperator;
  readonly args: any[];

  constructor(operator: PolarOperator, args: any[]) {
    this.operator = operator;
    this.args = args;
  }
}
