import { truncate as _truncate } from 'fs';

import type { Polar } from '../src/Polar';
import { Predicate } from '../src/Predicate';
import type { obj } from '../src/types';

type Result = Map<string, any>;

export async function query<T extends Polar>(
  x: T,
  q: string | Predicate
): Promise<Result[]> {
  const results = [];
  for await (const result of x.query(q)) {
    results.push(result);
  }
  return results;
}

export async function queryRule<T extends Polar>(
  x: T,
  name: string,
  ...args: any[]
): Promise<Result[]> {
  const results = [];
  for await (const result of x.queryRule(name, ...args)) {
    results.push(result);
  }
  return results;
}

export async function qvar<T extends Polar>(
  x: T,
  q: string | Predicate,
  prop: string,
  one?: boolean
): Promise<any> {
  const results = await query(x, q);
  return one ? results[0]?.get(prop) : results.map(r => r.get(prop));
}

export function pred(name: string, ...args: unknown[]): Predicate {
  return new Predicate(name, args);
}

export function map(obj?: obj): Map<string, any> {
  return new Map(Object.entries(obj || {}));
}

export function tempFile(contents: string, name?: string): Promise<string> {
  return require('temp-write')(contents, name);
}

export function tempFileFx(): Promise<string> {
  return tempFile('f(1);f(2);f(3);', 'f.polar');
}

export function tempFileGx(): Promise<string> {
  return tempFile('g(1);g(2);g(3);', 'g.polar');
}

export function truncate(file: string): Promise<string> {
  return new Promise((res, rej) =>
    _truncate(file, err => (err === null ? res() : rej(err)))
  );
}
