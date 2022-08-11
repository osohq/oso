"use strict";
var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Polar_ffiPolar, _Polar_host;
Object.defineProperty(exports, "__esModule", { value: true });
exports.Polar = void 0;
const path_1 = require("path");
const readline_1 = require("readline");
const repl_1 = require("repl");
const errors_1 = require("./errors");
const Query_1 = require("./Query");
const Host_1 = require("./Host");
const polar_wasm_api_1 = require("./polar_wasm_api");
const Predicate_1 = require("./Predicate");
const messages_1 = require("./messages");
const helpers_1 = require("./helpers");
class Source {
    constructor(src, filename) {
        this.src = src;
        this.filename = filename;
    }
}
/** Create and manage an instance of the Polar runtime. */
class Polar {
    constructor(opts = {}) {
        /**
         * Internal WebAssembly module.
         *
         * @internal
         */
        _Polar_ffiPolar.set(this, void 0);
        /**
         * Manages registration and comparison of JavaScript classes and instances
         * as well as translations between Polar and JavaScript values.
         *
         * @internal
         */
        _Polar_host.set(this, void 0);
        __classPrivateFieldSet(this, _Polar_ffiPolar, new polar_wasm_api_1.Polar(), "f");
        // This is a hack to make the POLAR_IGNORE_NO_ALLOW_WARNING env
        // variable accessible in wasm.
        __classPrivateFieldGet(this, _Polar_ffiPolar, "f").setIgnoreNoAllowWarning(!!(process === null || process === void 0 ? void 0 : process.env.POLAR_IGNORE_NO_ALLOW_WARNING));
        __classPrivateFieldSet(this, _Polar_host, new Host_1.Host(__classPrivateFieldGet(this, _Polar_ffiPolar, "f"), {
            acceptExpression: false,
            equalityFn: opts.equalityFn || helpers_1.defaultEqualityFn,
        }), "f");
        // Register global constants.
        this.registerConstant(null, 'nil');
        // Register built-in classes.
        this.registerClass(Boolean);
        this.registerClass(Number, { name: 'Integer' });
        this.registerClass(Number, { name: 'Float' });
        this.registerClass(String);
        this.registerClass(Array, { name: 'List' });
        this.registerClass(Map, { name: 'Dictionary' });
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
        __classPrivateFieldGet(this, _Polar_ffiPolar, "f").free();
    }
    /**
     * Process messages received from the Polar VM.
     *
     * @internal
     */
    processMessages() {
        for (;;) {
            const msg = __classPrivateFieldGet(this, _Polar_ffiPolar, "f").nextMessage();
            if (msg === undefined)
                break;
            messages_1.processMessage(msg);
        }
    }
    /**
     * Clear rules from the Polar KB, but
     * retain all registered classes and constants.
     */
    clearRules() {
        __classPrivateFieldGet(this, _Polar_ffiPolar, "f").clearRules();
        this.processMessages();
    }
    /**
     * Load Polar policy files.
     */
    async loadFiles(filenames) {
        if (filenames.length === 0)
            return;
        if (!path_1.extname) {
            throw new errors_1.PolarError('loadFiles is not supported in the browser');
        }
        const sources = await Promise.all(filenames.map(async (filename) => {
            if (path_1.extname(filename) !== '.polar')
                throw new errors_1.PolarFileExtensionError(filename);
            try {
                const contents = await helpers_1.readFile(filename);
                return new Source(contents, filename);
            }
            catch (e) {
                if (e.code === 'ENOENT')
                    throw new errors_1.PolarFileNotFoundError(filename);
                throw e;
            }
        }));
        return this.loadSources(sources);
    }
    /**
     * Load a Polar policy file.
     *
     * @deprecated `Oso.loadFile` has been deprecated in favor of `Oso.loadFiles`
     * as of the 0.20 release. Please see changelog for migration instructions:
     * https://docs.osohq.com/project/changelogs/2021-09-15.html
     */
    async loadFile(filename) {
        console.error('`Oso.loadFile` has been deprecated in favor of `Oso.loadFiles` as of the 0.20 release.\n\n' +
            'Please see changelog for migration instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html');
        return this.loadFiles([filename]);
    }
    /**
     * Load a Polar policy string.
     */
    async loadStr(contents, filename) {
        return this.loadSources([new Source(contents, filename)]);
    }
    // Register MROs, load Polar code, and check inline queries.
    async loadSources(sources) {
        __classPrivateFieldGet(this, _Polar_ffiPolar, "f").load(sources);
        this.processMessages();
        return this.checkInlineQueries();
    }
    async checkInlineQueries() {
        for (;;) {
            const query = __classPrivateFieldGet(this, _Polar_ffiPolar, "f").nextInlineQuery();
            this.processMessages();
            if (query === undefined)
                break;
            const source = query.source();
            const { results } = new Query_1.Query(query, this.getHost());
            const { done } = await results.next();
            await results.return();
            if (done)
                throw new errors_1.InlineQueryFailedError(source);
        }
    }
    /**
     * Query for a Polar predicate or string.
     */
    query(q, opts) {
        const host = Host_1.Host.clone(this.getHost(), {
            acceptExpression: (opts === null || opts === void 0 ? void 0 : opts.acceptExpression) || false,
        });
        let ffiQuery;
        if (helpers_1.isString(q)) {
            ffiQuery = __classPrivateFieldGet(this, _Polar_ffiPolar, "f").newQueryFromStr(q);
        }
        else {
            const term = host.toPolar(q);
            ffiQuery = __classPrivateFieldGet(this, _Polar_ffiPolar, "f").newQueryFromTerm(term);
        }
        this.processMessages();
        return new Query_1.Query(ffiQuery, host, opts === null || opts === void 0 ? void 0 : opts.bindings).results;
    }
    queryRule(nameOrOpts, ...args) {
        if (typeof nameOrOpts === 'string')
            return this.query(new Predicate_1.Predicate(nameOrOpts, args), {});
        if (typeof args[0] !== 'string')
            throw new errors_1.PolarError('Invalid call of queryRule(): missing rule name');
        const [ruleName, ...ruleArgs] = args;
        return this.query(new Predicate_1.Predicate(ruleName, ruleArgs), nameOrOpts);
    }
    /**
     * Query for a Polar rule, returning true if there are any results.
     */
    async queryRuleOnce(name, ...args) {
        const results = this.query(new Predicate_1.Predicate(name, args));
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
    registerClass(cls, params) {
        const clsName = this.getHost().cacheClass(cls, params);
        this.registerConstant(cls, clsName);
        this.getHost().registerMros();
    }
    /**
     * Register a JavaScript value for use in Polar policies.
     */
    registerConstant(value, name) {
        const term = this.getHost().toPolar(value);
        __classPrivateFieldGet(this, _Polar_ffiPolar, "f").registerConstant(name, term);
    }
    getHost() {
        return __classPrivateFieldGet(this, _Polar_host, "f");
    }
    getFfi() {
        return __classPrivateFieldGet(this, _Polar_ffiPolar, "f");
    }
    /** Start a REPL session. */
    async repl(files) {
        var _a;
        if (typeof readline_1.createInterface !== 'function')
            throw new errors_1.PolarError('REPL is not supported in the browser');
        try {
            if (files === null || files === void 0 ? void 0 : files.length)
                await this.loadFiles(files);
        }
        catch (e) {
            helpers_1.printError(e);
        }
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const repl = (_a = global.repl) === null || _a === void 0 ? void 0 : _a.repl; // eslint-disable-line @typescript-eslint/no-unsafe-member-access
        if (repl) {
            repl.setPrompt(helpers_1.PROMPT);
            const evalQuery = this.evalReplInput.bind(this);
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            repl.eval = async (evalCmd, _ctx, _file, cb) => cb(null, await evalQuery(evalCmd));
            const listeners = repl.listeners('exit');
            repl.removeAllListeners('exit');
            repl.prependOnceListener('exit', () => {
                listeners.forEach(l => repl.addListener('exit', l));
                repl_1.start({ useGlobal: true });
            });
        }
        else {
            const rl = readline_1.createInterface({
                input: process.stdin,
                output: process.stdout,
                prompt: helpers_1.PROMPT,
                tabSize: 4,
            });
            rl.prompt();
            // eslint-disable-next-line @typescript-eslint/no-misused-promises
            rl.on('line', async (line) => {
                const result = await this.evalReplInput(line);
                if (result !== undefined)
                    console.log(result);
                rl.prompt();
            });
        }
    }
    /**
     * Evaluate REPL input.
     *
     * @internal
     */
    async evalReplInput(query) {
        const input = query.trim().replace(/;+$/, '');
        try {
            if (input !== '') {
                const ffiQuery = __classPrivateFieldGet(this, _Polar_ffiPolar, "f").newQueryFromStr(input);
                const query = new Query_1.Query(ffiQuery, this.getHost());
                const results = [];
                for await (const result of query.results) {
                    results.push(result);
                }
                if (results.length === 0) {
                    return false;
                }
                else {
                    for (const result of results) {
                        for (const [variable, value] of result) {
                            console.log(variable + ' = ' + helpers_1.repr(value));
                        }
                    }
                    return true;
                }
            }
        }
        catch (e) {
            helpers_1.printError(e);
        }
    }
}
exports.Polar = Polar;
_Polar_ffiPolar = new WeakMap(), _Polar_host = new WeakMap();
//# sourceMappingURL=Polar.js.map