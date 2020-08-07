import { readFile } from 'fs/promises';
import { createHash } from 'crypto';
import { extname } from 'path';
import { createInterface } from 'readline';

import { Polar as FfiPolar } from '../../../polar-wasm-api/pkg/index';
import { Host } from './Host';
import { Query } from './Query';
import {
  InlineQueryFailedError,
  PolarError,
  PolarFileAlreadyLoadedError,
  PolarFileContentsChangedError,
  PolarFileDuplicateContentError,
  PolarFileExtensionError,
  PolarFileNotFoundError,
} from './errors';

export class Polar {
  #ffiPolar: FfiPolar;
  #host: Host;
  #loadedContents: Map<string, string>;
  #loadedNames: Map<string, string>;

  constructor() {
    this.#ffiPolar = new FfiPolar();
    this.#host = new Host(this.#ffiPolar);
    this.#loadedContents = new Map();
    this.#loadedNames = new Map();

    this.registerClass(Boolean);
    this.registerClass(Number);
    this.registerClass(String);
    this.registerClass(Array);
    this.registerClass(Object);
    // TODO(gj): should we register more than this? Map/Set? Function? Math/Date? JSON?
  }

  clear() {
    this.#loadedContents.clear();
    this.#loadedNames.clear();
    const previous = this.#ffiPolar;
    this.#ffiPolar = new FfiPolar();
    previous.free();
  }

  async loadFile(name: string): Promise<void> {
    if (extname(name) !== '.polar') throw new PolarFileExtensionError(name);
    let contents;
    try {
      contents = await readFile(name, { encoding: 'utf8' });
    } catch (e) {
      if (e.code === 'ENOENT') throw new PolarFileNotFoundError(name);
      throw e;
    }
    const hash = createHash('md5').update(contents).digest('hex');
    const matchingName = this.#loadedNames.get(name);
    if (matchingName !== undefined) {
      if (matchingName !== hash) throw new PolarFileContentsChangedError(name);
      throw new PolarFileAlreadyLoadedError(name);
    }
    const matchingContents = this.#loadedContents.get(hash);
    if (matchingContents !== undefined)
      throw new PolarFileDuplicateContentError(name, matchingContents);
    this.loadStr(contents, name);
    this.#loadedContents.set(name, hash);
    this.#loadedNames.set(hash, name);
  }

  private loadStr(contents: string, name?: string): void {
    this.#ffiPolar.loadFile(contents, name);
    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      if (query === undefined) break;
      const { done } = new Query(query, this.#host).results.next();
      if (done) throw new InlineQueryFailedError(name);
    }
  }

  query(query: Predicate | string): QueryResult {
    const host = this.#host.dup();
    let q;
    if (typeof query === 'string') {
      q = this.#ffiPolar.newQueryFromStr(query);
    } else {
      const term = JSON.stringify(host.toPolarTerm(query));
      q = this.#ffiPolar.newQueryFromTerm(term);
    }
    return new Query(q, host).results;
  }

  queryRule(name: string, args: unknown[]): QueryResult {
    return this.query(new Predicate(name, args));
  }

  repl(load: boolean): void {
    if (load) process.argv.slice(2).forEach(this.loadFile);
    createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: 'query> ',
      tabSize: 4,
    }).on('line', line => {
      const input = line.trim().replace(/;+$/, '');
      try {
        const ffiQuery = this.#ffiPolar.newQueryFromStr(input);
        const results = Array.from(new Query(ffiQuery, this.#host).results);
        if (results.length === 0) {
          console.log(false);
        } else {
          for (const result of results) {
            if (result.size === 0) {
              console.log(true);
            } else {
              console.log(JSON.stringify(result, null, 4));
            }
          }
        }
      } catch (e) {
        if (e.kind.split('::')[0] === 'ParseError') {
          console.log(`Parse error: ${e}`);
        } else if (e instanceof PolarError) {
          console.log(e);
        } else {
          throw e;
        }
      }
    });
  }

  // TODO(gj): is Function the most accurate type here?
  registerClass(cls: Function, alias?: string, ctor?: Constructor): void {
    const name = this.#host.cacheClass(cls, alias, ctor);
    this.registerConstant(name, cls);
  }

  registerConstant(name: string, value: any): void {
    const term = JSON.stringify(this.#host.toPolarTerm(value));
    this.#ffiPolar.registerConstant(name, term);
  }
}
