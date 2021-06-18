import type { obj } from './types';

/** Polar pattern. */
export class Pattern {
  readonly tag?: string;
  readonly fields: Map<string, unknown>;

  constructor({ tag, fields }: { tag: string; fields: Map<string, unknown> }) {
    this.tag = tag;
    this.fields = fields || {};
  }
}
