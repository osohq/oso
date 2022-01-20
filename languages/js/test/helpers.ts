import write from 'temp-write';

import type { Polar } from '../src/Polar';
import { Predicate } from '../src/Predicate';
import type { obj, QueryOpts } from '../src/types';

type Result = Map<string, unknown>;

export async function query<Q, R>(
  x: Polar<Q, R>,
  q: string | Predicate,
  opts?: QueryOpts
): Promise<Result[]> {
  const results = [];
  for await (const result of x.query(q, opts)) {
    results.push(result);
  }
  return results;
}

export async function queryRule<Q, R>(
  x: Polar<Q, R>,
  name: string,
  ...args: unknown[]
): Promise<Result[]> {
  const results = [];
  for await (const result of x.queryRule(name, ...args)) {
    results.push(result);
  }
  return results;
}

export async function qvar<Q, R>(
  x: Polar<Q, R>,
  q: string | Predicate,
  prop: string,
  one?: boolean
): Promise<unknown> {
  const results = await query(x, q);
  return one ? results[0]?.get(prop) : results.map(r => r.get(prop));
}

export function pred(name: string, ...args: unknown[]): Predicate {
  return new Predicate(name, args);
}

export function map(obj?: obj): Map<string, unknown> {
  return new Map(Object.entries(obj || {}));
}

export function tempFile(contents: string, name?: string): Promise<string> {
  return write(contents, name);
}

export function tempFileFx(): Promise<string> {
  return tempFile('f(1);f(2);f(3);', 'f.polar');
}

export function tempFileGx(): Promise<string> {
  return tempFile('g(1);g(2);g(3);', 'g.polar');
}
