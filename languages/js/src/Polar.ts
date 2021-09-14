const extname = require('path')?.extname;
const createInterface = require('readline')?.createInterface;

import {
  InlineQueryFailedError,
  PolarError,
  PolarFileExtensionError,
  PolarFileNotFoundError,
  DuplicateClassAliasError,
} from './errors';
import { Query } from './Query';
import { Host, UserType } from './Host';
import { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import { processMessage } from './messages';
import { Class, obj, Options, QueryResult } from './types';
import { printError, PROMPT, readFile, repr } from './helpers';

import { Variable } from './Variable';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import { serializeTypes, filterData } from './dataFiltering';

class Source {
  readonly src: string;
  readonly filename?: string;

  constructor(src: string, filename?: string) {
    this.src = src;
    this.filename = filename;
  }
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

  constructor(opts: Options = {}) {
    function defaultEqual(a: any, b: any) {
      if (
        a &&
        b && // good grief!!
        typeof a === typeof b &&
        typeof a === 'object' &&
        a.__proto__ === b.__proto__
      ) {
        let check = new Map();

        for (let x in a) {
          if (!defaultEqual(a[x], b[x])) return false;
          check.set(x, true);
        }

        for (let x in b) if (!check.get(x)) return false;

        return true;
      }
      return a == b;
    }

    this.#ffiPolar = new FfiPolar();
    const equalityFn = opts.equalityFn || defaultEqual;
    this.#host = new Host(this.#ffiPolar, equalityFn);

    // Register global constants.
    this.registerConstant(null, 'nil');

    // Register built-in classes.
    this.registerClass(Boolean);
    this.registerClass(Number, { name: 'Integer' });
    this.registerClass(Number, { name: 'Float' });
    this.registerClass(String);
    this.registerClass(Array, { name: 'List' });
    this.registerClass(Object, { name: 'Dictionary' });
  }

  /**
   * Free the underlying WASM instance.
   *
   * Invariant: ensure that you do *not* do anything else with an instance
   * after calling `free()` on it.
   *
   * This should *not* be something you need to do during the course of regular
   * usage. It's generally only useful for scenarios where large numbers of
   * instances are spun up and not cleanly reaped by the GC, such as during a
   * long-running test process in 'watch' mode.
   */
  free() {
    this.#ffiPolar.free();
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
   * Clear rules from the Polar KB, but
   * retain all registered classes and constants.
   */
  clearRules() {
    this.#ffiPolar.clearRules();
    this.processMessages();
  }

  /**
   * Load Polar policy files.
   */
  async loadFiles(filenames: string[]): Promise<void> {
    if (filenames.length === 0) return;

    if (!extname) {
      throw new PolarError('loadFiles is not supported in the browser');
    }
    const sources = await Promise.all(
      filenames.map(async filename => {
        if (extname(filename) !== '.polar')
          throw new PolarFileExtensionError(filename);

        try {
          const contents = await readFile(filename);
          return new Source(contents, filename);
        } catch (e) {
          if (e.code === 'ENOENT') throw new PolarFileNotFoundError(filename);
          throw e;
        }
      })
    );

    return this.loadSources(sources);
  }

  /**
   * Load a Polar policy file.
   *
   * @deprecated `Oso.loadFile` has been deprecated in favor of `Oso.loadFiles`
   * as of the 0.20.0 release. Please see changelog for migration instructions:
   * https://docs.osohq.com/project/changelogs/2021-09-15.html
   */
  async loadFile(filename: string): Promise<void> {
    console.error(
      '`Oso.loadFile` has been deprecated in favor of `Oso.loadFiles` as of the 0.20.0 release.\n\n' +
        'Please see changelog for migration instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html'
    );
    return this.loadFiles([filename]);
  }

  /**
   * Load a Polar policy string.
   */
  async loadStr(contents: string, filename?: string): Promise<void> {
    return this.loadSources([new Source(contents, filename)]);
  }

  // Register MROs, load Polar code, and check inline queries.
  private async loadSources(sources: Source[]): Promise<void> {
    this.#host.registerMros();
    this.#ffiPolar.load(sources);
    this.processMessages();
    return this.checkInlineQueries();
  }

  private async checkInlineQueries(): Promise<void> {
    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      this.processMessages();
      if (query === undefined) break;
      const source = query.source();
      const { results } = new Query(query, this.#host);
      const { done } = await results.next();
      await results.return();
      if (done) throw new InlineQueryFailedError(source);
    }
  }

  /**
   * Query for a Polar predicate or string.
   */
  query(q: Predicate | string, bindings?: Map<string, any>): QueryResult {
    const host = Host.clone(this.#host);
    let ffiQuery;
    if (typeof q === 'string') {
      ffiQuery = this.#ffiPolar.newQueryFromStr(q);
    } else {
      const term = JSON.stringify(host.toPolar(q));
      ffiQuery = this.#ffiPolar.newQueryFromTerm(term);
    }
    this.processMessages();
    return new Query(ffiQuery, host, bindings).results;
  }

  /**
   * Query for a Polar rule with bindings.
   */
  queryRuleWithBindings(
    name: string,
    bindings: Map<string, any>,
    ...args: unknown[]
  ): QueryResult {
    return this.query(new Predicate(name, args), bindings);
  }

  /**
   * Query for a Polar rule.
   */
  queryRule(name: string, ...args: unknown[]): QueryResult {
    return this.query(new Predicate(name, args));
  }

  /**
   * Query for a Polar rule, returning true if there are any results.
   */
  async queryRuleOnce(name: string, ...args: unknown[]): Promise<boolean> {
    const results = this.query(new Predicate(name, args));
    const { done } = await results.next();
    await results.return();
    return !done;
  }

  /**
   * Register a JavaScript class for use in Polar policies.
   *
   * @param cls The class to register.
   * @param params An optional object with extra parameters.
   *
   * Accepted extra parameters are:
   * - name: Explicit name to use for the class in Polar.
   * - types: A map or object of string keys to type values, used for data filtering.
   * - buildQuery: A function to produce a query for `cls` objects
   * - execQuery: A function to execute a query produced by `buildQuery`
   * - combineQuery: A function to merge two queries produced by `buildQuery`
   */
  registerClass<T>(cls: Class<T>, params?: any): void {
    const clsName = this.#host.cacheClass(cls, params);
    this.registerConstant(cls, clsName);
  }

  /**
   * Register a JavaScript value for use in Polar policies.
   */
  registerConstant(value: any, name: string): void {
    const term = this.#host.toPolar(value);
    this.#ffiPolar.registerConstant(name, JSON.stringify(term));
  }

  getHost(): Host {
    return this.#host;
  }

  getFfi(): FfiPolar {
    return this.#ffiPolar;
  }

  /** Start a REPL session. */
  async repl(files?: string[]): Promise<void> {
    if (createInterface == null) {
      throw new PolarError('REPL is not supported in the browser');
    }
    try {
      if (files?.length) await this.loadFiles(files);
    } catch (e) {
      printError(e);
    }

    // @ts-ignore
    const repl = global.repl?.repl;

    if (repl) {
      repl.setPrompt(PROMPT);
      const evalQuery = this.evalReplInput.bind(this);
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
        output: process.stdout,
        prompt: PROMPT,
        tabSize: 4,
      });
      rl.prompt();
      rl.on('line', async (line: string) => {
        const result = await this.evalReplInput(line);
        if (result !== undefined) console.log(result);
        rl.prompt();
      });
    }
  }

  /**
   * Evaluate REPL input.
   *
   * @internal
   */
  private async evalReplInput(query: string): Promise<boolean | void> {
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
              console.log(variable + ' = ' + repr(value));
            }
          }
          return true;
        }
      }
    } catch (e) {
      printError(e);
    }
  }
}
