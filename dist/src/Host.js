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
var _Host_ffiPolar, _Host_instances, _Host_opts;
Object.defineProperty(exports, "__esModule", { value: true });
exports.Host = void 0;
const errors_1 = require("./errors");
const helpers_1 = require("./helpers");
const Expression_1 = require("./Expression");
const Pattern_1 = require("./Pattern");
const Predicate_1 = require("./Predicate");
const Variable_1 = require("./Variable");
const types_1 = require("./types");
const types_2 = require("./types");
const filter_1 = require("./filter");
/**
 * Translator between Polar and JavaScript.
 *
 * @internal
 */
class Host {
    /** @internal */
    constructor(ffiPolar, opts) {
        _Host_ffiPolar.set(this, void 0);
        _Host_instances.set(this, void 0);
        _Host_opts.set(this, void 0);
        __classPrivateFieldSet(this, _Host_ffiPolar, ffiPolar, "f");
        __classPrivateFieldSet(this, _Host_opts, opts, "f");
        __classPrivateFieldSet(this, _Host_instances, new Map(), "f");
        this.types = new Map();
        this.adapter = {
            buildQuery: () => {
                throw new errors_1.DataFilteringConfigurationError();
            },
            executeQuery: () => {
                throw new errors_1.DataFilteringConfigurationError();
            },
        };
    }
    /**
     * Shallow clone a host to extend its state for the duration of a particular
     * query without modifying the longer-lived [[`Polar`]] host state.
     *
     * @internal
     */
    static clone(host, opts) {
        const options = { ...__classPrivateFieldGet(host, _Host_opts, "f"), ...opts };
        const clone = new Host(__classPrivateFieldGet(host, _Host_ffiPolar, "f"), options);
        __classPrivateFieldSet(clone, _Host_instances, new Map(__classPrivateFieldGet(host, _Host_instances, "f")), "f");
        clone.types = new Map(host.types);
        clone.adapter = host.adapter;
        return clone;
    }
    /**
     * Fetch a JavaScript class from the class cache.
     *
     * @param name Class name to look up.
     *
     * @internal
     */
    getClass(name) {
        const typ = this.types.get(name);
        if (typ === undefined)
            throw new errors_1.UnregisteredClassError(name);
        return typ.cls;
    }
    /**
     * Get user type for `cls`.
     *
     * @param cls Class or class name.
     */
    getType(cls) {
        if (cls === undefined)
            return undefined;
        return this.types.get(cls);
    }
    /**
     * Return user types that are registered with Host.
     */
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    *distinctUserTypes() {
        for (const [name, typ] of this.types)
            if (helpers_1.isString(name))
                yield typ;
    }
    serializeTypes() {
        var _a;
        const polarTypes = {};
        for (const [tag, userType] of this.types) {
            if (helpers_1.isString(tag)) {
                const fields = userType.fields;
                const fieldTypes = {};
                for (const [k, v] of fields) {
                    if (v instanceof filter_1.Relation) {
                        fieldTypes[k] = v.serialize();
                    }
                    else {
                        const class_tag = (_a = this.getType(v)) === null || _a === void 0 ? void 0 : _a.name;
                        if (class_tag === undefined)
                            throw new errors_1.UnregisteredClassError(v.name);
                        fieldTypes[k] = { Base: { class_tag } };
                    }
                }
                polarTypes[tag] = fieldTypes;
            }
        }
        return polarTypes;
    }
    /**
     * Store a JavaScript class in the class cache.
     *
     * @param cls Class to cache.
     * @param params Optional parameters.
     *
     * @internal
     */
    cacheClass(cls, params) {
        params = params ? params : {};
        // TODO(gw) maybe we only want to support plain objects?
        let fields = params.fields || {};
        if (!(fields instanceof Map))
            fields = new Map(Object.entries(fields));
        const { name } = params;
        if (!helpers_1.isConstructor(cls))
            throw new errors_1.InvalidConstructorError(cls);
        const clsName = name ? name : cls.name;
        const existing = this.types.get(clsName);
        if (existing) {
            throw new errors_1.DuplicateClassAliasError({
                name: clsName,
                cls,
                existing: existing.cls,
            });
        }
        function defaultCheck(instance) {
            return instance instanceof cls || (instance === null || instance === void 0 ? void 0 : instance.constructor) === cls;
        }
        const userType = new types_1.UserType({
            name: clsName,
            cls,
            fields,
            id: this.cacheInstance(cls),
            isaCheck: params.isaCheck || defaultCheck,
        });
        this.types.set(cls, userType);
        this.types.set(clsName, userType);
        return clsName;
    }
    /**
     * Return cached instances.
     *
     * Only used by the test suite.
     *
     * @internal
     */
    instances() {
        return Array.from(__classPrivateFieldGet(this, _Host_instances, "f").values());
    }
    /**
     * Check if an instance exists in the instance cache.
     *
     * @internal
     */
    hasInstance(id) {
        return __classPrivateFieldGet(this, _Host_instances, "f").has(id);
    }
    /**
     * Fetch a JavaScript instance from the instance cache.
     *
     * Public for the test suite.
     *
     * @internal
     */
    getInstance(id) {
        if (!this.hasInstance(id))
            throw new errors_1.UnregisteredInstanceError(id);
        return __classPrivateFieldGet(this, _Host_instances, "f").get(id);
    }
    /**
     * Store a JavaScript instance in the instance cache, fetching a new instance
     * ID from the Polar VM if an ID is not provided.
     *
     * @internal
     */
    cacheInstance(instance, id) {
        let instanceId = id;
        if (instanceId === undefined) {
            instanceId = __classPrivateFieldGet(this, _Host_ffiPolar, "f").newId();
        }
        __classPrivateFieldGet(this, _Host_instances, "f").set(instanceId, instance);
        return instanceId;
    }
    /**
     * Register the MROs of all registered classes.
     */
    registerMros() {
        // Get MRO of all registered classes
        // NOTE: not ideal that the MRO gets updated each time loadStr is
        // called, but since we are planning to move to only calling load once
        // with the include feature, I think it's okay for now.
        for (const typ of this.distinctUserTypes()) {
            // Get MRO for type.
            const mro = helpers_1.ancestors(typ.cls)
                .map(c => { var _a; return (_a = this.getType(c)) === null || _a === void 0 ? void 0 : _a.id; })
                .filter(id => id !== undefined);
            // Register with core.
            __classPrivateFieldGet(this, _Host_ffiPolar, "f").registerMro(typ.name, mro);
        }
    }
    /**
     * Construct a JavaScript instance and store it in the instance cache.
     *
     * @internal
     */
    async makeInstance(name, fields, id) {
        const cls = this.getClass(name);
        const args = await Promise.all(fields.map(f => this.toJs(f)));
        const instance = new cls(...args);
        this.cacheInstance(instance, id);
    }
    /**
     * Check if the left class is more specific than the right class with respect
     * to the given instance.
     *
     * @internal
     */
    async isSubspecializer(id, left, right) {
        let instance = this.getInstance(id);
        instance = instance instanceof Promise ? await instance : instance; // eslint-disable-line @typescript-eslint/no-unsafe-assignment
        const mro = helpers_1.ancestors(instance === null || instance === void 0 ? void 0 : instance.constructor);
        const leftIndex = mro.indexOf(this.getClass(left));
        const rightIndex = mro.indexOf(this.getClass(right));
        if (leftIndex === -1) {
            return false;
        }
        else if (rightIndex === -1) {
            return true;
        }
        else {
            return leftIndex < rightIndex;
        }
    }
    /**
     * Check if the left class is a subclass of the right class.
     *
     * @internal
     */
    isSubclass(left, right) {
        const leftCls = this.getClass(left);
        const rightCls = this.getClass(right);
        const mro = helpers_1.ancestors(leftCls);
        return mro.includes(rightCls);
    }
    /**
     * Check if the given instance is an instance of a particular class.
     *
     * @internal
     */
    async isa(polarInstance, name) {
        const instance = (await this.toJs(polarInstance));
        const userType = this.types.get(name);
        if (userType !== undefined) {
            return userType.isaCheck(instance);
        }
        else {
            const cls = this.getClass(name);
            return instance instanceof cls || (instance === null || instance === void 0 ? void 0 : instance.constructor) === cls;
        }
    }
    /**
     * Check if a sequence of field accesses on the given class is an
     * instance of another class.
     *
     * @internal
     */
    async isaWithPath(baseTag, path, classTag) {
        var _a;
        let tag = baseTag;
        for (const fld of path) {
            const field = await this.toJs(fld);
            if (!helpers_1.isString(field))
                throw new Error(`Not a field name: ${helpers_1.repr(field)}`);
            const userType = this.types.get(tag);
            if (userType === undefined)
                return false;
            let fieldType = userType.fields.get(field);
            if (fieldType === undefined)
                return false;
            if (fieldType instanceof filter_1.Relation) {
                switch (fieldType.kind) {
                    case 'one': {
                        const otherCls = (_a = this.getType(fieldType.otherType)) === null || _a === void 0 ? void 0 : _a.cls;
                        if (otherCls === undefined)
                            throw new errors_1.UnregisteredClassError(fieldType.otherType);
                        fieldType = otherCls;
                        break;
                    }
                    case 'many':
                        fieldType = Array;
                        break;
                }
            }
            const newBase = this.getType(fieldType);
            if (newBase === undefined)
                return false;
            tag = newBase.name;
        }
        return classTag === tag;
    }
    /**
     * Check if the given instances conform to the operator.
     *
     * @internal
     */
    async externalOp(op, leftTerm, rightTerm) {
        // NOTE(gj): These are `any` because JS puts no type boundaries on what's
        // comparable. Want to resolve `{} > NaN` to an arbitrary boolean? Go nuts!
        const left = (await this.toJs(leftTerm)); // eslint-disable-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
        const right = (await this.toJs(rightTerm)); // eslint-disable-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
        switch (op) {
            case 'Eq':
                return __classPrivateFieldGet(this, _Host_opts, "f").equalityFn(left, right);
            case 'Geq':
                return left >= right;
            case 'Gt':
                return left > right;
            case 'Leq':
                return left <= right;
            case 'Lt':
                return left < right;
            case 'Neq':
                return !__classPrivateFieldGet(this, _Host_opts, "f").equalityFn(left, right);
            default: {
                const _ = op;
                return _;
            }
        }
    }
    /**
     * Turn a JavaScript value into a Polar term that's ready to be sent to the
     * Polar VM.
     *
     * @internal
     */
    toPolar(v) {
        var _a, _b, _c;
        switch (true) {
            case typeof v === 'boolean':
                return { value: { Boolean: v } };
            case Number.isInteger(v):
                return { value: { Number: { Integer: v } } };
            case typeof v === 'number':
                if (v === Infinity) {
                    v = 'Infinity';
                }
                else if (v === -Infinity) {
                    v = '-Infinity';
                }
                else if (Number.isNaN(v)) {
                    v = 'NaN';
                }
                return { value: { Number: { Float: v } } };
            case helpers_1.isString(v):
                return { value: { String: v } };
            case Array.isArray(v): {
                const polarTermList = v.map(a => this.toPolar(a));
                return { value: { List: polarTermList } };
            }
            case v instanceof Predicate_1.Predicate: {
                const { name, args } = v;
                const polarArgs = args.map(a => this.toPolar(a));
                return { value: { Call: { name, args: polarArgs } } };
            }
            case v instanceof Variable_1.Variable:
                return { value: { Variable: v.name } };
            case v instanceof Expression_1.Expression: {
                const { operator, args } = v;
                const polarArgs = args.map(a => this.toPolar(a));
                return { value: { Expression: { operator, args: polarArgs } } };
            }
            case v instanceof Pattern_1.Pattern: {
                const { tag, fields } = v;
                let dict = this.toPolar(fields).value;
                // TODO(gj): will `dict.Dictionary` ever be undefined?
                if (!types_2.isPolarDict(dict))
                    dict = { Dictionary: { fields: new Map() } };
                if (tag === undefined)
                    return { value: { Pattern: dict } };
                return {
                    value: { Pattern: { Instance: { tag, fields: dict.Dictionary } } },
                };
            }
            case v instanceof types_2.Dict: {
                const fields = new Map(Object.entries(v).map(([k, v]) => [k, this.toPolar(v)]));
                return { value: { Dictionary: { fields } } };
            }
            default: {
                let instanceId = undefined;
                let classId = undefined;
                // pass a string class repr *for registered types only*, otherwise pass
                // undefined (allow core to differentiate registered or not)
                const v_cast = v;
                let classRepr = undefined;
                if (helpers_1.isConstructor(v)) {
                    instanceId = (_a = this.getType(v)) === null || _a === void 0 ? void 0 : _a.id;
                    classRepr = "Class";
                }
                else {
                    const v_constructor = v_cast === null || v_cast === void 0 ? void 0 : v_cast.constructor;
                    // pass classId for instances of *registered classes* only
                    if (v_constructor !== undefined && this.types.has(v_constructor)) {
                        classId = (_b = this.getType(v_constructor)) === null || _b === void 0 ? void 0 : _b.id;
                        classRepr = (_c = this.getType(v_constructor)) === null || _c === void 0 ? void 0 : _c.name;
                    }
                }
                // pass classRepr for *registered* classes only, pass undefined
                // otherwise
                if (classRepr !== undefined && !this.types.has(classRepr)) {
                    classRepr = undefined;
                }
                // cache it if not already cached
                instanceId = instanceId || this.cacheInstance(v);
                return {
                    value: {
                        ExternalInstance: {
                            instance_id: instanceId,
                            constructor: undefined,
                            repr: helpers_1.repr(v),
                            class_repr: classRepr,
                            class_id: classId,
                        },
                    },
                };
            }
        }
    }
    /**
     * Turn a Polar term from the Polar VM into a JavaScript value.
     *
     * @internal
     */
    async toJs(v) {
        const t = v.value;
        if (types_2.isPolarStr(t)) {
            return t.String;
        }
        else if (types_2.isPolarNum(t)) {
            if ('Float' in t.Number) {
                const f = t.Number.Float;
                switch (f) {
                    case 'Infinity':
                        return Infinity;
                    case '-Infinity':
                        return -Infinity;
                    case 'NaN':
                        return NaN;
                    default:
                        if (typeof f !== 'number')
                            throw new errors_1.PolarError('Expected a floating point number, got "' + f + '"');
                        return f;
                }
            }
            else {
                return t.Number.Integer;
            }
        }
        else if (types_2.isPolarBool(t)) {
            return t.Boolean;
        }
        else if (types_2.isPolarList(t)) {
            return await Promise.all(t.List.map(async (el) => await this.toJs(el)));
        }
        else if (types_2.isPolarDict(t)) {
            const valueToJs = ([k, v]) => this.toJs(v).then(v => [k, v]);
            const { fields } = t.Dictionary;
            const entries = await Promise.all([...fields.entries()].map(valueToJs));
            return entries.reduce((dict, [k, v]) => {
                dict[k] = v;
                return dict;
            }, new types_2.Dict());
        }
        else if (types_2.isPolarInstance(t)) {
            const i = this.getInstance(t.ExternalInstance.instance_id);
            return i instanceof Promise ? await i : i; // eslint-disable-line @typescript-eslint/no-unsafe-return
        }
        else if (types_2.isPolarPredicate(t)) {
            const { name, args } = t.Call;
            const jsArgs = await Promise.all(args.map(a => this.toJs(a)));
            return new Predicate_1.Predicate(name, jsArgs);
        }
        else if (types_2.isPolarVariable(t)) {
            return new Variable_1.Variable(t.Variable);
        }
        else if (types_2.isPolarExpression(t)) {
            if (!__classPrivateFieldGet(this, _Host_opts, "f").acceptExpression)
                throw new errors_1.UnexpectedExpressionError();
            const { operator, args: argTerms } = t.Expression;
            const args = await Promise.all(argTerms.map(a => this.toJs(a)));
            return new Expression_1.Expression(operator, args);
        }
        else if (types_2.isPolarPattern(t)) {
            if ('Dictionary' in t.Pattern) {
                const fields = (await this.toJs({ value: t.Pattern }));
                return new Pattern_1.Pattern({ fields });
            }
            else {
                const { tag, fields: { fields }, } = t.Pattern.Instance;
                const dict = await this.toJs({ value: { Dictionary: { fields } } });
                return new Pattern_1.Pattern({ tag, fields: dict });
            }
        }
        else {
            const _ = t;
            return _;
        }
    }
}
exports.Host = Host;
_Host_ffiPolar = new WeakMap(), _Host_instances = new WeakMap(), _Host_opts = new WeakMap();
//# sourceMappingURL=Host.js.map