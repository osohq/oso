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
var _Oso_notFoundError, _Oso_forbiddenError, _Oso_readAction;
Object.defineProperty(exports, "__esModule", { value: true });
exports.Oso = void 0;
const Polar_1 = require("./Polar");
const Variable_1 = require("./Variable");
const Expression_1 = require("./Expression");
const Pattern_1 = require("./Pattern");
const errors_1 = require("./errors");
const filter_1 = require("./filter");
/** The Oso authorization API. */
// TODO(gj): maybe pass DF options to constructor & try to parametrize a
// `Query` type w/ the return type of the provided buildQuery fn.
class Oso extends Polar_1.Polar {
    constructor(opts = {}) {
        super(opts);
        _Oso_notFoundError.set(this, errors_1.NotFoundError);
        _Oso_forbiddenError.set(this, errors_1.ForbiddenError);
        _Oso_readAction.set(this, 'read');
        if (opts.notFoundError)
            __classPrivateFieldSet(this, _Oso_notFoundError, opts.notFoundError, "f");
        if (opts.forbiddenError)
            __classPrivateFieldSet(this, _Oso_forbiddenError, opts.forbiddenError, "f");
        if (opts.readAction)
            __classPrivateFieldSet(this, _Oso_readAction, opts.readAction, "f");
    }
    /**
     * Query the knowledge base to determine whether an actor is allowed to
     * perform an action upon a resource.
     *
     * @param actor Subject.
     * @param action Verb.
     * @param resource Object.
     * @returns An access control decision.
     */
    async isAllowed(actor, action, resource) {
        return this.queryRuleOnce('allow', actor, action, resource);
    }
    /**
     * Ensure that `actor` is allowed to perform `action` on
     * `resource`.
     *
     * If the action is permitted with an `allow` rule in the policy, then
     * this method returns `None`. If the action is not permitted by the
     * policy, this method will raise an error.
     *
     * The error raised by this method depends on whether the actor can perform
     * the `"read"` action on the resource. If they cannot read the resource,
     * then a `NotFound` error is raised. Otherwise, a `ForbiddenError` is
     * raised.
     *
     * @param actor The actor performing the request.
     * @param action The action the actor is attempting to perform.
     * @param resource The resource being accessed.
     * @param checkRead If set to `false`, a `ForbiddenError` is always
     *   thrown on authorization failures, regardless of whether the actor can
     *   read the resource. Default is `true`.
     */
    async authorize(actor, action, resource, options = {}) {
        if (typeof options.checkRead === 'undefined')
            options.checkRead = true;
        if (await this.queryRuleOnce('allow', actor, action, resource)) {
            return;
        }
        let isNotFound = false;
        if (options.checkRead) {
            if (action === __classPrivateFieldGet(this, _Oso_readAction, "f")) {
                isNotFound = true;
            }
            else {
                const canRead = await this.queryRuleOnce('allow', actor, __classPrivateFieldGet(this, _Oso_readAction, "f"), resource);
                if (!canRead) {
                    isNotFound = true;
                }
            }
        }
        const ErrorClass = isNotFound ? __classPrivateFieldGet(this, _Oso_notFoundError, "f") : __classPrivateFieldGet(this, _Oso_forbiddenError, "f");
        throw new ErrorClass();
    }
    /**
     * Determine the actions `actor` is allowed to take on `resource`.
     *
     * Collects all actions allowed by allow rules in the Polar policy for the
     * given combination of actor and resource.
     *
     * @param actor The actor for whom to collect allowed actions
     * @param resource The resource being accessed
     * @param allowWildcard Flag to determine behavior if the policy
     *   includes a wildcard action. E.g., a rule allowing any action:
     *   `allow(_actor, _action, _resource)`. If `true`, the method will
     *   return `["*"]`, if `false`, the method will raise an exception.
     * @returns A list of the unique allowed actions.
     */
    async authorizedActions(actor, resource, options = {}) {
        const results = this.queryRule('allow', actor, new Variable_1.Variable('action'), resource);
        const actions = new Set();
        for await (const result of results) {
            const action = result.get('action');
            if (action instanceof Variable_1.Variable) {
                if (!options.allowWildcard) {
                    throw new errors_1.OsoError(`
            The result of authorizedActions() contained an "unconstrained" action that could represent any action, but allowWildcard was set to False. To fix, set allowWildcard to True and compare with the "*" string.
          `);
                }
                else {
                    return new Set(['*']);
                }
            }
            // TODO(gj): do we need to handle the case where `action` is something
            // other than a `Variable` or an `Action`? E.g., if it's an `Expression`?
            actions.add(action);
        }
        return actions;
    }
    /**
     * Ensure that `actor` is allowed to send `request` to the server.
     *
     * Checks the `allow_request` rule of a policy.
     *
     * If the request is permitted with an `allow_request` rule in the
     * policy, then this method returns nothing. Otherwise, this method raises
     * a `ForbiddenError`.
     *
     * @param actor The actor performing the request.
     * @param request An object representing the request that was sent by the
     *   actor.
     */
    async authorizeRequest(actor, request) {
        const isAllowed = await this.queryRuleOnce('allow_request', actor, request);
        if (!isAllowed) {
            throw new (__classPrivateFieldGet(this, _Oso_forbiddenError, "f"))();
        }
    }
    /**
     * Ensure that `actor` is allowed to perform `action` on a given
     * `resource`'s `field`.
     *
     * If the action is permitted by an `allow_field` rule in the policy,
     * then this method returns nothing. If the action is not permitted by the
     * policy, this method will raise a `ForbiddenError`.
     *
     * @param actor The actor performing the request.
     * @param action The action the actor is attempting to perform on the
     * field.
     * @param resource The resource being accessed.
     * @param field The name of the field being accessed.
     */
    async authorizeField(actor, action, resource, field) {
        const isAllowed = await this.queryRuleOnce('allow_field', actor, action, resource, field);
        if (!isAllowed) {
            throw new (__classPrivateFieldGet(this, _Oso_forbiddenError, "f"))();
        }
    }
    /**
     * Determine the fields of `resource` on which `actor` is allowed to
     * perform  `action`.
     *
     * Uses `allow_field` rules in the policy to find all allowed fields.
     *
     * @param actor The actor for whom to collect allowed fields.
     * @param action The action being taken on the field.
     * @param resource The resource being accessed.
     * @param allowWildcard Flag to determine behavior if the policy \
     *   includes a wildcard field. E.g., a rule allowing any field: \
     *   `allow_field(_actor, _action, _resource, _field)`. If `true`, the \
     *   method will return `["*"]`, if `false`, the method will raise an \
     *   exception.
     * @returns A list of the unique allowed fields.
     */
    async authorizedFields(actor, action, resource, options = {}) {
        const results = this.queryRule('allow_field', actor, action, resource, new Variable_1.Variable('field'));
        const fields = new Set();
        for await (const result of results) {
            const field = result.get('field');
            if (field instanceof Variable_1.Variable) {
                if (!options.allowWildcard) {
                    throw new errors_1.OsoError(`
            The result of authorizedFields() contained an "unconstrained" field that could represent any field, but allowWildcard was set to False. To fix, set allowWildcard to True and compare with the "*" string.
          `);
                }
                else {
                    return new Set(['*']);
                }
            }
            // TODO(gj): do we need to handle the case where `field` is something
            // other than a `Variable` or a `Field`? E.g., if it's an `Expression`?
            fields.add(field);
        }
        return fields;
    }
    /**
     * Create a query for all the resources of type `resourceCls` that `actor` is
     * allowed to perform `action` on.
     *
     * @param actor Subject.
     * @param action Verb.
     * @param resourceCls Object type.
     * @returns A query that selects authorized resources of type `resourceCls`
     */
    async authorizedQuery(actor, action, resourceCls) {
        var _a;
        const resource = new Variable_1.Variable('resource');
        const host = this.getHost();
        var clsName;
        if (typeof resourceCls === 'string') {
            clsName = resourceCls;
        }
        else {
            clsName = (_a = host.getType(resourceCls)) === null || _a === void 0 ? void 0 : _a.name;
            if (clsName === undefined)
                throw new errors_1.UnregisteredClassError(resourceCls.name);
        }
        const constraint = new Expression_1.Expression('And', [
            new Expression_1.Expression('Isa', [
                resource,
                new Pattern_1.Pattern({ tag: clsName, fields: {} }),
            ]),
        ]);
        const bindings = new Map();
        bindings.set('resource', constraint);
        const results = this.queryRule({
            bindings,
            acceptExpression: true,
        }, 'allow', actor, action, resource);
        const queryResults = [];
        for await (const result of results) {
            queryResults.push({
                // convert bindings back into Polar
                bindings: new Map([...result.entries()].map(([k, v]) => [k, host.toPolar(v)])),
            });
        }
        const dataFilter = this.getFfi().buildDataFilter(host.serializeTypes(), queryResults, 'resource', clsName);
        const filter = await filter_1.parseFilter(dataFilter, host);
        return host.adapter.buildQuery(filter);
    }
    /**
     * Determine the resources of type `resourceCls` that `actor`
     * is allowed to perform `action` on.
     *
     * @param actor Subject.
     * @param action Verb.
     * @param resourceCls Object type or string name of class
     * @returns An array of authorized resources.
     */
    async authorizedResources(actor, action, resourceCls) {
        const query = await this.authorizedQuery(actor, action, resourceCls);
        if (!query)
            return [];
        return this.getHost().adapter.executeQuery(query);
    }
    /**
     * Register adapter for data filtering query functions.
     */
    setDataFilteringAdapter(adapter) {
        this.getHost().adapter = adapter;
    }
}
exports.Oso = Oso;
_Oso_notFoundError = new WeakMap(), _Oso_forbiddenError = new WeakMap(), _Oso_readAction = new WeakMap();
//# sourceMappingURL=Oso.js.map