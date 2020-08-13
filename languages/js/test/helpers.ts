import type { Polar } from '../src/Polar';
import { Predicate } from '../src/Predicate';
import type { obj } from '../src/types';

type Result = Map<string, any>;

export function query(polar: Polar, q: string | Predicate): Result[] {
  return Array.from(polar.query(q));
}

export function qvar(
  polar: Polar,
  q: string | Predicate,
  prop: string,
  one?: boolean
): any {
  const results = query(polar, q);
  return one ? results[0]?.get(prop) : results.map(r => r.get(prop));
}

export function pred(name: string, ...args: unknown[]): Predicate {
  return new Predicate(name, args);
}

export function map(obj?: obj): Map<string, any> {
  return new Map(Object.entries(obj || {}));
}

export function tempFile(contents: string, name?: string): string {
  return require('temp-write').sync(contents, name);
}

export function tempFileFx(): string {
  return tempFile('f(1);f(2);f(3);', 'f.polar');
}

export function tempFileGx(): string {
  return tempFile('g(1);g(2);g(3);', 'g.polar');
}
