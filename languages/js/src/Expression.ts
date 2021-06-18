import type { PolarOperator } from './types';
// import { Pattern } from 'Pattern';

/** Polar expression. */
export class Expression {
  readonly operator: PolarOperator;
  readonly args: any[];

  constructor(operator: PolarOperator, args: any[]) {
    this.operator = operator;
    this.args = args;
  }
}

// /** Polar type constraint. */
// export class TypeConstraint extends Expression {
//   constructor(constrainee: unknown, tag: string) {
//     const isa = new Expression('Isa', [constrainee, new Pattern(tag)]);
//     super('And', [isa]);
//   }
// }
