/** Polar predicate. */
export class Predicate {
  readonly name: string;
  readonly args: unknown[];

  constructor(name: string, args: unknown[]) {
    this.name = name;
    this.args = args;
  }
}
