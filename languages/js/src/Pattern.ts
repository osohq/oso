import { Dict } from './types';

/** Polar pattern. */
export class Pattern {
  readonly tag?: string;
  readonly fields: Dict;

  constructor({ tag, fields }: { tag?: string; fields: Dict }) {
    this.tag = tag;
    this.fields = fields;
  }
}
