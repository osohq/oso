import { readFile } from 'fs/promises';
import { createHash } from 'crypto';
import { extname, isAbsolute, resolve } from 'path';
import { createInterface } from 'readline';

import {
  InlineQueryFailedError,
  PolarError,
  PolarFileAlreadyLoadedError,
  PolarFileContentsChangedError,
  PolarFileDuplicateContentError,
  PolarFileExtensionError,
  PolarFileNotFoundError,
} from './errors';
import { Query } from './Query';
import { Host } from './Host';
import { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import type { Class, Options, QueryResult } from './types';

export class Polar {
  #ffiPolar: FfiPolar;
  #host: Host;
  #loadedContents: Map<string, string>;
  #loadedFiles: Map<string, string>;

  constructor(opts: Options = {}) {
    this.#ffiPolar = new FfiPolar();
    const equalityFn = opts.equalityFn || ((x, y) => x == y);
    this.#host = new Host(this.#ffiPolar, equalityFn);
    this.#loadedContents = new Map();
    this.#loadedFiles = new Map();

    this.registerClass(Boolean);
    this.registerClass(Number, 'Integer');
    this.registerClass(Number, 'Float');
    this.registerClass(String);
    this.registerClass(Array, 'List');
    this.registerClass(Object, 'Dictionary');
  }

  // For tests only.
  __host() {
    return this.#host;
  }

  clear() {
    this.#loadedContents.clear();
    this.#loadedFiles.clear();
    const previous = this.#ffiPolar;
    this.#ffiPolar = new FfiPolar();
    previous.free();
  }

  async loadFile(name: string): Promise<void> {
    if (extname(name) !== '.polar') throw new PolarFileExtensionError(name);
    let file = isAbsolute(name) ? name : resolve(__dirname, name);
    let contents;
    try {
      contents = await readFile(file, { encoding: 'utf8' });
    } catch (e) {
      if (e.code === 'ENOENT') throw new PolarFileNotFoundError(file);
      throw e;
    }
    const hash = createHash('md5').update(contents).digest('hex');
    const existingContents = this.#loadedFiles.get(file);
    if (existingContents !== undefined) {
      if (existingContents === hash)
        throw new PolarFileAlreadyLoadedError(file);
      throw new PolarFileContentsChangedError(file);
    }
    const existingFile = this.#loadedContents.get(hash);
    if (existingFile !== undefined)
      throw new PolarFileDuplicateContentError(file, existingFile);
    await this.loadStr(contents, file);
    this.#loadedContents.set(hash, file);
    this.#loadedFiles.set(file, hash);
  }

  async loadStr(contents: string, name?: string): Promise<void> {
    this.#ffiPolar.loadFile(contents, name);
    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      if (query === undefined) break;
      const { results } = new Query(query, this.#host);
      const { done } = await results.next();
      results.return();
      if (done) throw new InlineQueryFailedError(name);
    }
  }

  query(q: Predicate | string): QueryResult {
    const host = Host.clone(this.#host);
    let ffiQuery;
    if (typeof q === 'string') {
      ffiQuery = this.#ffiPolar.newQueryFromStr(q);
    } else {
      const term = JSON.stringify(host.toPolar(q));
      ffiQuery = this.#ffiPolar.newQueryFromTerm(term);
    }
    return new Query(ffiQuery, host).results;
  }

  queryRule(name: string, ...args: unknown[]): QueryResult {
    return this.query(new Predicate(name, args));
  }

  repl(load: boolean): void {
    if (load) process.argv.slice(2).forEach(this.loadFile);
    createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: 'query> ',
      tabSize: 4,
    }).on('line', async line => {
      const input = line.trim().replace(/;+$/, '');
      try {
        const ffiQuery = this.#ffiPolar.newQueryFromStr(input);
        const query = new Query(ffiQuery, this.#host);
        const results = [];
        for await (const result of query.results) {
          results.push(result);
        }
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

  registerClass<T>(cls: Class<T>, alias?: string): void {
    const name = this.#host.cacheClass(cls, alias);
    this.registerConstant(name, cls);
  }

  registerConstant(name: string, value: any): void {
    const term = this.#host.toPolar(value);
    this.#ffiPolar.registerConstant(name, JSON.stringify(term));
  }
}
