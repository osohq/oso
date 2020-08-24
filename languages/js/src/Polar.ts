import { createHash } from 'crypto';
import { extname } from 'path';
import { createInterface } from 'readline';
import { stdout, stderr } from 'process';

import {
  InlineQueryFailedError,
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
import { processMessage } from './messages';
import type { Class, Options, QueryResult } from './types';
import { readFile, repr } from './helpers';

let RESET = '';
let FG_BLUE = '';
let FG_RED = '';
if (
  typeof stdout.hasColors === 'function' &&
  stdout.hasColors() &&
  typeof stderr.hasColors === 'function' &&
  stderr.hasColors()
) {
  RESET = '\x1b[0m';
  FG_BLUE = '\x1b[34m';
  FG_RED = '\x1b[31m';
}

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

  private processMessages() {
    while (true) {
      let msg = this.#ffiPolar.nextMessage();
      if (msg === undefined) break;
      processMessage(msg);
    }
  }

  clear() {
    this.#loadedContents.clear();
    this.#loadedFiles.clear();
    const previous = this.#ffiPolar;
    this.#ffiPolar = new FfiPolar();
    previous.free();
  }

  async loadFile(file: string): Promise<void> {
    if (extname(file) !== '.polar') throw new PolarFileExtensionError(file);
    let contents;
    try {
      contents = await readFile(file);
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
    this.processMessages();

    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      this.processMessages();
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
    this.processMessages();
    return new Query(ffiQuery, host).results;
  }

  queryRule(name: string, ...args: unknown[]): QueryResult {
    return this.query(new Predicate(name, args));
  }

  async repl(files?: string[]): Promise<void> {
    const rl = createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: FG_BLUE + 'query> ' + RESET,
      tabSize: 4,
    });

    let loadError;
    try {
      if (files?.length) await Promise.all(files.map(f => this.loadFile(f)));
    } catch (e) {
      loadError = e;
    }
    if (loadError !== undefined) {
      console.error(FG_RED + 'One or more files failed to load.' + RESET);
      console.error(loadError.message);
    }

    rl.prompt();
    rl.on('line', async line => {
      const input = line.trim().replace(/;+$/, '');
      try {
        if (input === '') return;
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
              for (const [variable, value] of result) {
                console.log(variable + ' => ' + repr(value));
              }
            }
          }
        }
      } catch (e) {
        console.error(FG_RED + e.name + RESET);
        console.error(e.message);
      } finally {
        rl.prompt();
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
