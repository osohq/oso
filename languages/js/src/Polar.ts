import { extname } from 'path';
import { createInterface } from 'readline';
import type { REPLServer } from 'repl';
import { start } from 'repl';
import { Context } from 'vm';

import {
  InlineQueryFailedError,
  PolarError,
  PolarFileExtensionError,
  PolarFileNotFoundError,
} from './errors';
import { Query } from './Query';
import { Host } from './Host';
import { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import type { Message } from './messages';
import { processMessage } from './messages';
import {
  Class,
  ClassParams,
  Dict,
  Options,
  QueryOpts,
  QueryResult,
} from './types';
import {
  defaultEqualityFn,
  isString,
  printError,
  PROMPT,
  readFile,
  repr,
} from './helpers';

class Source {
  readonly src: string;
  readonly filename?: string;

  constructor(src: string, filename?: string) {
    this.src = src;
    this.filename = filename;
  }
}

/** Create and manage an instance of the Polar runtime. */
export class Polar<Query, Resource> {
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
  #host: Host<Query, Resource>;

  constructor(opts: Options = {}) {
    this.#ffiPolar = new FfiPolar();
    // This is a hack to make the POLAR_IGNORE_NO_ALLOW_WARNING env
    // variable accessible in wasm.
    this.#ffiPolar.setIgnoreNoAllowWarning(
      !!process?.env.POLAR_IGNORE_NO_ALLOW_WARNING
    );
    this.#host = new Host(this.#ffiPolar, {
      acceptExpression: false,
      equalityFn: opts.equalityFn || defaultEqualityFn,
    });

    // Register global constants.
    this.registerConstant(null, 'nil');

    // Register built-in classes.
    this.registerClass(Boolean);
    this.registerClass(Number, { name: 'Integer' });
    this.registerClass(Number, { name: 'Float' });
    this.registerClass(String);
    this.registerClass(Array, { name: 'List' });
    this.registerClass(Dict, { name: 'Dictionary' });
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
  free(): void {
    this.#ffiPolar.free();
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private processMessages() {
    for (;;) {
      const msg = this.#ffiPolar.nextMessage() as Message | undefined;
      if (msg === undefined) break;
      processMessage(msg);
    }
  }

  /**
   * Clear rules from the Polar KB, but
   * retain all registered classes and constants.
   */
  clearRules(): void {
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
          if ((e as NodeJS.ErrnoException).code === 'ENOENT')
            throw new PolarFileNotFoundError(filename);
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
   * as of the 0.20 release. Please see changelog for migration instructions:
   * https://docs.osohq.com/project/changelogs/2021-09-15.html
   */
  async loadFile(filename: string): Promise<void> {
    console.error(
      '`Oso.loadFile` has been deprecated in favor of `Oso.loadFiles` as of the 0.20 release.\n\n' +
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
    this.#ffiPolar.load(sources);
    this.processMessages();
    return this.checkInlineQueries();
  }

  private async checkInlineQueries(): Promise<void> {
    for (;;) {
      const query = this.#ffiPolar.nextInlineQuery();
      this.processMessages();
      if (query === undefined) break;
      const source = query.source();
      const { results } = new Query(query, this.getHost());
      const { done } = await results.next();
      await results.return();
      if (done) throw new InlineQueryFailedError(source);
    }
  }

  /**
   * Query for a Polar predicate or string.
   */
  query(q: Predicate | string, opts?: QueryOpts): QueryResult {
    const host = Host.clone(this.getHost(), {
      acceptExpression: opts?.acceptExpression || false,
    });

    let ffiQuery;
    if (isString(q)) {
      ffiQuery = this.#ffiPolar.newQueryFromStr(q);
    } else {
      const term = host.toPolar(q);
      ffiQuery = this.#ffiPolar.newQueryFromTerm(term);
    }
    this.processMessages();
    return new Query(ffiQuery, host, opts?.bindings).results;
  }

  /**
   * Query for a Polar rule.
   */
  queryRule(opts: QueryOpts, name: string, ...args: unknown[]): QueryResult;
  queryRule(name: string, ...args: unknown[]): QueryResult;
  queryRule(nameOrOpts: string | QueryOpts, ...args: unknown[]): QueryResult {
    if (typeof nameOrOpts === 'string')
      return this.query(new Predicate(nameOrOpts, args), {});

    if (typeof args[0] !== 'string')
      throw new PolarError('Invalid call of queryRule(): missing rule name');

    const [ruleName, ...ruleArgs] = args;
    return this.query(new Predicate(ruleName, ruleArgs), nameOrOpts);
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
   */
  registerClass(cls: Class, params?: ClassParams): void {
    const clsName = this.getHost().cacheClass(cls, params);
    this.registerConstant(cls, clsName);
    this.getHost().registerMros();
  }

  /**
   * Register a JavaScript value for use in Polar policies.
   */
  registerConstant(value: unknown, name: string): void {
    const term = this.getHost().toPolar(value);
    this.#ffiPolar.registerConstant(name, term);
  }

  getHost(): Host<Query, Resource> {
    return this.#host;
  }

  getFfi(): FfiPolar {
    return this.#ffiPolar;
  }

  /** Start a REPL session. */
  async repl(files?: string[]): Promise<void> {
    if (typeof createInterface !== 'function')
      throw new PolarError('REPL is not supported in the browser');

    try {
      if (files?.length) await this.loadFiles(files);
    } catch (e) {
      printError(e as Error);
    }

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    const repl = global.repl?.repl as REPLServer | undefined; // eslint-disable-line @typescript-eslint/no-unsafe-member-access

    if (repl) {
      repl.setPrompt(PROMPT);
      const evalQuery = this.evalReplInput.bind(this);
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      repl.eval = async (
        evalCmd: string,
        _ctx: Context,
        _file: string,
        cb: (err: Error | null, result: boolean | void) => void
      ) => cb(null, await evalQuery(evalCmd));
      const listeners = repl.listeners('exit') as (() => void)[];
      repl.removeAllListeners('exit');
      repl.prependOnceListener('exit', () => {
        listeners.forEach(l => repl.addListener('exit', l));
        start({ useGlobal: true });
      });
    } else {
      const rl = createInterface({
        input: process.stdin,
        output: process.stdout,
        prompt: PROMPT,
        tabSize: 4,
      });
      rl.prompt();
      // eslint-disable-next-line @typescript-eslint/no-misused-promises
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
        const query = new Query(ffiQuery, this.getHost());
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
      printError(e as Error);
    }
  }
}
