const extname = require('path')?.extname;
const createInterface = require('readline')?.createInterface;

import {
  InlineQueryFailedError,
  InvalidConstructorError,
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
import { Class, Dict, obj, Options, QueryResult } from './types';
import { isConstructor, printError, PROMPT, readFile, repr } from './helpers';
import { Variable } from './Variable';
import { Expression } from './Expression';
import type { PolarOperator } from './types';
import { Pattern } from './Pattern';
import { serializeTypes, filterData } from './dataFiltering';
import { assert } from 'console';

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
    this.#rolesEnabled = false;

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
      await results.return();
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

  configureDataFiltering({ buildQuery, execQuery, combineQuery }: any) {
    if (buildQuery) this.#host.buildQuery = buildQuery;
    if (execQuery) this.#host.execQuery = execQuery;
    if (combineQuery) this.#host.combineQuery = combineQuery;
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
   */
  registerClass<T>(cls: Class<T>, params?: any): void {
    params = params ? params : {};
    const { name, types, buildQuery, execQuery, combineQuery } = params;
    if (!isConstructor(cls)) throw new InvalidConstructorError(cls);
    const clsName = name ? name : cls.name;
    const existing = this.#host.types.get(clsName);
    if (existing) {
      throw new DuplicateClassAliasError({
        name: clsName,
        cls,
        existing,
      });
    }
    const userType = new UserType({
      name: clsName,
      class: cls,
      buildQuery: buildQuery || this.#host.buildQuery,
      execQuery: execQuery || this.#host.execQuery,
      combineQuery: combineQuery || this.#host.combineQuery,
      fields: types || new Map(),
    });
    this.#host.types.set(cls, userType);
    this.#host.types.set(clsName, userType);
    this.registerConstant(cls, clsName);
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
  async authorizedQuery(actor: any, action: any, cls: any): Promise<any> {
    const resource = new Variable('resource');
    const clsName = this.#host.types.get(cls)!.name;
    const constraint = new Expression('And', [
      new Expression('Isa', [
        resource,
        new Pattern({ tag: clsName, fields: {} }),
      ]),
    ]);
    let bindings = new Map();
    bindings.set('resource', constraint);
    let results = this.queryRuleWithBindings(
      'allow',
      bindings,
      actor,
      action,
      resource
    );

    const queryResults = [];
    for await (const result of results) {
      queryResults.push(result);
    }

    let jsonResults = queryResults.map(result => ({
      // `Map<string, any> -> {[key: string]: PolarTerm}` b/c Maps aren't
      // trivially `JSON.stringify()`-able.
      bindings: [...result.entries()].reduce((obj: obj, [k, v]) => {
        obj[k] = this.#host.toPolar(v);
        return obj;
      }, {}),
    }));
    let resultsStr = JSON.stringify(jsonResults);
    let typesStr = serializeTypes(this.#host.types);
    let plan = this.#ffiPolar.buildFilterPlan(
      typesStr,
      resultsStr,
      'resource',
      clsName
    );
    return filterData(this.#host, plan);
  }

  async authorizedResources(actr: any, actn: any, cls: any): Promise<any> {
    const query = await this.authorizedQuery(actr, actn, cls);
    return !query ? [] : this.#host.types.get(cls)!.execQuery!(query);
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
