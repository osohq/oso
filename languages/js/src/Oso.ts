import { Polar } from './Polar';
import { Variable } from './Variable';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import type {
  Options,
  CustomError,
  obj,
  Class,
  PolarTerm,
  DataFilteringQueryParams,
} from './types';
import {
  NotFoundError,
  ForbiddenError,
  OsoError,
  UnregisteredClassError,
} from './errors';
import { filterData } from './dataFiltering';
import type { FilterPlan } from './dataFiltering';

/** The Oso authorization API. */
// TODO(gj): maybe pass DF options to constructor & try to parametrize a
// `Query` type w/ the return type of the provided buildQuery fn.
export class Oso<
  Actor = unknown,
  Action = unknown,
  Resource = unknown,
  Field = unknown,
  Request = unknown
> extends Polar {
  #notFoundError: CustomError = NotFoundError;
  #forbiddenError: CustomError = ForbiddenError;
  #readAction: unknown = 'read';

  constructor(opts: Options = {}) {
    super(opts);

    if (opts.notFoundError) this.#notFoundError = opts.notFoundError;
    if (opts.forbiddenError) this.#forbiddenError = opts.forbiddenError;
    if (opts.readAction) this.#readAction = opts.readAction;
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
  async isAllowed(
    actor: Actor,
    action: Action,
    resource: Resource
  ): Promise<boolean> {
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
  async authorize(
    actor: Actor,
    action: Action,
    resource: Resource,
    options: { checkRead?: boolean } = {}
  ): Promise<void> {
    if (typeof options.checkRead === 'undefined') options.checkRead = true;
    if (await this.queryRuleOnce('allow', actor, action, resource)) {
      return;
    }

    let isNotFound = false;
    if (options.checkRead) {
      if (action === this.#readAction) {
        isNotFound = true;
      } else {
        const canRead = await this.queryRuleOnce(
          'allow',
          actor,
          this.#readAction,
          resource
        );
        if (!canRead) {
          isNotFound = true;
        }
      }
    }
    const ErrorClass = isNotFound ? this.#notFoundError : this.#forbiddenError;
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
  async authorizedActions(
    actor: Actor,
    resource: Resource,
    options: { allowWildcard?: boolean } = {}
  ): Promise<Set<Action | '*'>> {
    const results = this.queryRule(
      'allow',
      actor,
      new Variable('action'),
      resource
    );
    const actions = new Set<Action | '*'>();
    for await (const result of results) {
      const action = result.get('action');
      if (action instanceof Variable) {
        if (!options.allowWildcard) {
          throw new OsoError(`
            The result of authorizedActions() contained an "unconstrained" action that could represent any action, but allowWildcard was set to False. To fix, set allowWildcard to True and compare with the "*" string.
          `);
        } else {
          return new Set(['*']);
        }
      }
      // TODO(gj): do we need to handle the case where `action` is something
      // other than a `Variable` or an `Action`? E.g., if it's an `Expression`?
      actions.add(action as Action);
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
  async authorizeRequest(actor: Actor, request: Request): Promise<void> {
    const isAllowed = await this.queryRuleOnce('allow_request', actor, request);
    if (!isAllowed) {
      throw new this.#forbiddenError();
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
  async authorizeField(
    actor: Actor,
    action: Action,
    resource: Resource,
    field: Field
  ): Promise<void> {
    const isAllowed = await this.queryRuleOnce(
      'allow_field',
      actor,
      action,
      resource,
      field
    );
    if (!isAllowed) {
      throw new this.#forbiddenError();
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
  async authorizedFields(
    actor: Actor,
    action: Action,
    resource: Resource,
    options: { allowWildcard?: boolean } = {}
  ): Promise<Set<Field | '*'>> {
    const results = this.queryRule(
      'allow_field',
      actor,
      action,
      resource,
      new Variable('field')
    );
    const fields = new Set<Field | '*'>();
    for await (const result of results) {
      const field = result.get('field');
      if (field instanceof Variable) {
        if (!options.allowWildcard) {
          throw new OsoError(`
            The result of authorizedFields() contained an "unconstrained" field that could represent any field, but allowWildcard was set to False. To fix, set allowWildcard to True and compare with the "*" string.
          `);
        } else {
          return new Set(['*']);
        }
      }
      // TODO(gj): do we need to handle the case where `field` is something
      // other than a `Variable` or a `Field`? E.g., if it's an `Expression`?
      fields.add(field as Field);
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
  async authorizedQuery(
    actor: Actor,
    action: Action,
    resourceCls: Class<Resource>
  ): Promise<unknown> {
    const resource = new Variable('resource');
    const host = this.getHost();
    const clsName = host.getType(resourceCls)?.name;
    if (clsName === undefined)
      throw new UnregisteredClassError(resourceCls.name);

    const constraint = new Expression('And', [
      new Expression('Isa', [
        resource,
        new Pattern({ tag: clsName, fields: {} }),
      ]),
    ]);
    const bindings = new Map();
    bindings.set('resource', constraint);
    const results = this.queryRuleWithBindings(
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

    const jsonResults = queryResults.map(result => ({
      // `Map<string, unknown> -> {[key: string]: PolarTerm}` b/c Maps aren't
      // trivially `JSON.stringify()`-able.
      bindings: [...result.entries()].reduce((obj: obj<PolarTerm>, [k, v]) => {
        obj[k] = host.toPolar(v);
        return obj;
      }, {}),
    }));
    const resultsStr = JSON.stringify(jsonResults);
    const plan = this.getFfi().buildFilterPlan(
      host.serializeTypes(),
      resultsStr,
      'resource',
      clsName
    );
    return filterData(host, plan as FilterPlan);
  }

  /**
   * Determine the resources of type `resourceCls` that `actor`
   * is allowed to perform `action` on.
   *
   * @param actor Subject.
   * @param action Verb.
   * @param resourceCls Object type.
   * @returns An array of authorized resources.
   */
  async authorizedResources<T extends Resource>(
    actor: Actor,
    action: Action,
    resourceCls: Class<T>
  ): Promise<T[]> {
    const query = await this.authorizedQuery(actor, action, resourceCls);
    if (!query) return [];
    const userType = this.getHost().getType(resourceCls);
    if (userType === undefined)
      throw new UnregisteredClassError(resourceCls.name);
    return userType.execQuery(query);
  }

  /**
   * Register default values for data filtering query functions.
   * These can be overridden by passing specific implementations to
   * `registerClass`.
   */
  setDataFilteringQueryDefaults(options: DataFilteringQueryParams) {
    if (options.buildQuery) this.getHost().buildQuery = options.buildQuery;
    if (options.execQuery) this.getHost().execQuery = options.execQuery;
    if (options.combineQuery)
      this.getHost().combineQuery = options.combineQuery;
  }
}
