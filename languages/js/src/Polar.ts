const extname = require('path')?.extname;
const createInterface = require('readline')?.createInterface;

import {
  InlineQueryFailedError,
  InvalidConstructorError,
  PolarError,
  PolarFileExtensionError,
  PolarFileNotFoundError,
} from './errors';
import { Query } from './Query';
import { Host } from './Host';
import { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import { processMessage } from './messages';
import type { Class, obj, Options, QueryResult } from './types';
import { isConstructor, printError, PROMPT, readFile, repr } from './helpers';
import { Variable } from './Variable';

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
   * Flag that tracks if the roles feature is enabled.
   *
   * @internal
   */
  #rolesEnabled: boolean;

  constructor(opts: Options = {}) {
    this.#ffiPolar = new FfiPolar();
    const equalityFn = opts.equalityFn || ((x, y) => x == y);
    this.#host = new Host(this.#ffiPolar, equalityFn);
    this.#rolesEnabled = false;

    // Register global constants.
    this.registerConstant(null, 'nil');

    // Register built-in classes.
    this.registerClass(Boolean);
    this.registerClass(Number, 'Integer');
    this.registerClass(Number, 'Float');
    this.registerClass(String);
    this.registerClass(Array, 'List');
    this.registerClass(Object, 'Dictionary');
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
   * Enable Oso's built-in roles feature.
   */
  async enableRoles() {
    if (!this.#rolesEnabled) {
      const helpers = {
        join: (sep: string, l: string, r: string) => [l, r].join(sep),
      };
      this.registerConstant(helpers, '__oso_internal_roles_helpers__');
      this.#ffiPolar.enableRoles();
      this.processMessages();
      await this.validateRolesConfig();
      this.#rolesEnabled = true;
    }
  }

  /**
   * Validate roles config.
   *
   * @internal
   */
  private async validateRolesConfig() {
    const validationQueryResults = [];
    while (true) {
      const query = this.#ffiPolar.nextInlineQuery();
      this.processMessages();
      if (query === undefined) break;
      const { results } = new Query(query, this.#host);
      const queryResults = [];
      for await (const result of results) {
        queryResults.push(result);
      }
      validationQueryResults.push(queryResults);
    }

    const results = validationQueryResults.map(results =>
      results.map(result => ({
        // `Map<string, any> -> {[key: string]: PolarTerm}` b/c Maps aren't
        // trivially `JSON.stringify()`-able.
        bindings: [...result.entries()].reduce((obj: obj, [k, v]) => {
          obj[k] = this.#host.toPolar(v);
          return obj;
        }, {}),
      }))
    );

    this.#ffiPolar.validateRolesConfig(JSON.stringify(results));
    this.processMessages();
  }

  /**
   * Clear rules from the Polar KB, but
   * retain all registered classes and constants.
   */
  clearRules() {
    this.#ffiPolar.clearRules();
    this.processMessages();
    this.#rolesEnabled = false;
  }

  /**
   * Load a Polar policy file.
   */
  async loadFile(file: string): Promise<void> {
    if (!extname) {
      throw new PolarError('loadFile is not supported in the browser');
    }
    if (extname(file) !== '.polar') throw new PolarFileExtensionError(file);
    let contents;
    try {
      contents = await readFile(file);
    } catch (e) {
      if (e.code === 'ENOENT') throw new PolarFileNotFoundError(file);
      throw e;
    }
    await this.loadStr(contents, file);
  }

  /**
   * Load a Polar policy string.
   */
  async loadStr(contents: string, name?: string): Promise<void> {
    this.#ffiPolar.load(contents, name);
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

    if (this.#rolesEnabled) {
      this.#rolesEnabled = false;
      await this.enableRoles();
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
   * Register a JavaScript class for use in Polar policies.
   */
  registerClass<T>(cls: Class<T>, alias?: string, types?: Map<string, any>, fetcher?: any): void {
    if (!isConstructor(cls)) throw new InvalidConstructorError(cls);
    const clsName = this.#host.cacheClass(cls, alias);
    this.registerConstant(cls, clsName);
    this.#host.clsNames.set(cls, clsName);
    if (types != null) {
      this.#host.types.set(clsName, types);
    }
    if (fetcher != null) {
      this.#host.fetchers.set(clsName, fetcher);
    }
  }

  /**
   * Register a JavaScript value for use in Polar policies.
   */
  registerConstant(value: any, name: string): void {
    const term = this.#host.toPolar(value);
    this.#ffiPolar.registerConstant(name, JSON.stringify(term));
  }

  /**
   * Returns all the resources the actor is allowed to perform some action on.
   */
  getAllowedResources(actor: any, action: any, cls: any): any {
    const resource = new Variable("resource");
    // @TODO: register class stuff to get class name

  }

  /** Start a REPL session. */
  async repl(files?: string[]): Promise<void> {
    if (createInterface == null) {
      throw new PolarError('REPL is not supported in the browser');
    }
    try {
      if (files?.length) await Promise.all(files.map(f => this.loadFile(f)));
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
