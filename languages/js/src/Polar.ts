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

// Optional ANSI escape sequences for the REPL.
let RESET = '';
let FG_BLUE = '';
let FG_RED = '';
if (stdout.getColorDepth() >= 4 && stderr.getColorDepth() >= 4) {
  RESET = '\x1b[0m';
  FG_BLUE = '\x1b[34m';
  FG_RED = '\x1b[31m';
}

/** Create and manage an instance of the Polar runtime. */
export class Polar {
  /**
   * Internal WebAssembly module.
   *
   * @internal
   */
  #ffiPolar: FfiPolar;
  /**
   * Manages registration and comparison of JavaScript classes and instances
   * as well as translations between Polar and JavaScript values.
   *
   * @internal
   */
  #host: Host;
  /**
   * Tracking Polar files loaded into the knowledge base by a hash of their
   * contents.
   *
   * @internal
   */
  #loadedContents: Map<string, string>;
  /**
   * Tracking Polar files loaded into the knowledge base by file name.
   *
   * @internal
   */
  #loadedFiles: Map<string, string>;

  constructor(opts: Options = {}) {
    this.#ffiPolar = new FfiPolar();
    const equalityFn = opts.equalityFn || ((x, y) => x == y);
    this.#host = new Host(this.#ffiPolar, equalityFn);
    this.#loadedContents = new Map();
    this.#loadedFiles = new Map();

    // Register built-in classes.
    this.registerClass(Boolean);
    this.registerClass(Number, 'Integer');
    this.registerClass(Number, 'Float');
    this.registerClass(String);
    this.registerClass(Array, 'List');
    this.registerClass(Object, 'Dictionary');
  }

  /**
   * For tests only.
   *
   * @hidden
   */
  __host() {
    return this.#host;
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private processMessages() {
    while (true) {
      let msg = this.#ffiPolar.nextMessage();
      if (msg === undefined) break;
      processMessage(msg);
    }
  }

  /**
   * Replace the current Polar VM instance, clearing out all loaded policy but
   * retaining all registered classes and constants.
   */
  clear() {
    this.#loadedContents.clear();
    this.#loadedFiles.clear();
    const previous = this.#ffiPolar;
    this.#ffiPolar = new FfiPolar();
    previous.free();
  }

  /**
   * Load a Polar policy file.
   */
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

  /**
   * Load a Polar policy string.
   */
  async loadStr(contents: string, name?: string): Promise<void> {
    this.#ffiPolar.loadFile(contents, name);
    this.processMessages();

    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      this.processMessages();
      if (query === undefined) break;
      const source = query.source();
      const { results } = new Query(query, this.#host);
      const { done } = await results.next();
      results.return();
      if (done) throw new InlineQueryFailedError(source);
    }
  }

  /**
   * Query for a Polar predicate or string.
   */
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

  /**
   * Query for a Polar rule.
   */
  queryRule(name: string, ...args: unknown[]): QueryResult {
    return this.query(new Predicate(name, args));
  }

  /**
   * Start a REPL session.
   */
  async repl(files?: string[]): Promise<void> {
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

    // @ts-ignore
    const repl = global.repl?.repl;

    if (repl) {
      repl.setPrompt(FG_BLUE + 'query> ' + RESET);
      const evalQuery = this.evalQuery.bind(this);
      repl.eval = async (cmd: string, _ctx: any, _file: string, cb: Function) =>
        cb(null, await evalQuery(cmd));
      const listeners: Function[] = repl.listeners('exit');
      repl.removeAllListeners('exit');
      repl.prependOnceListener('exit', () => {
        listeners.forEach(l => repl.addListener('exit', l));
        require('repl').start({ useGlobal: true });
      });
    } else {
      const rl = createInterface({
        input: process.stdin,
        output: stdout,
        prompt: FG_BLUE + 'query> ' + RESET,
        tabSize: 4,
      });

      rl.prompt();
      rl.on('line', async line => {
        const result = await this.evalQuery(line);
        console.log(result);
        rl.prompt();
      });
    }
  }

  private async evalQuery(query: string): Promise<boolean | void> {
    const input = query.trim().replace(/;+$/, '');
    try {
      if (input !== '') {
        const ffiQuery = this.#ffiPolar.newQueryFromStr(input);
        const query = new Query(ffiQuery, this.#host);
        const results = [];
        for await (const result of query.results) {
          results.push(result);
        }
        if (results.length === 0) {
          return false;
        } else {
          for (const result of results) {
            for (const [variable, value] of result) {
              console.log(variable + ' => ' + repr(value));
            }
          }
          return true;
        }
      }
    } catch (e) {
      console.error(FG_RED + e.name + RESET);
      console.error(e.message);
    }
  }

  /**
   * Register a JavaScript class for use in Polar policies.
   */
  registerClass<T>(cls: Class<T>, alias?: string): void {
    const name = this.#host.cacheClass(cls, alias);
    this.registerConstant(name, cls);
  }

  /**
   * Register a JavaScript value for use in Polar policies.
   */
  registerConstant(name: string, value: any): void {
    const term = this.#host.toPolar(value);
    this.#ffiPolar.registerConstant(name, JSON.stringify(term));
  }
}
