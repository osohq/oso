import { ForbiddenError, NotFoundError, OsoError } from './errors';
import { Policy } from './Oso';
import { Variable } from './Variable';

type CustomError = new (...args: any[]) => Error;

export interface EnforcerOptions<Action> {
  /**
   * Optionally override the "not found" error class thrown by `authorize`.
   * Defaults to {@link NotFoundError}.
   */
  notFoundError?: CustomError;
  /**
   * Optionally override the "forbidden" error class thrown by the `authorize*`
   * methods. Defaults to {@link ForbiddenError}.
   */
  forbiddenError?: CustomError;
  /**
   * The action used by the `authorize` method to determine whether an
   * authorization failure should raise a `NotFoundError` or a `ForbiddenError`.
   */
  readAction?: Action;
}

function defaultGetError(isNotFound: boolean) {
  if (isNotFound) return new NotFoundError();
  return new ForbiddenError();
}

/**
 * NOTE: This is a preview feature.
 *
 * Exposes high-level enforcement APIs which can be used by apps to perform
 * resource-, request-, and query-level authorization.
 */
export class Enforcer<
  Actor = unknown,
  Action = unknown,
  Resource = unknown,
  Field = unknown,
  Request = unknown
> {
  policy: Policy;
  #notFoundError: CustomError = NotFoundError;
  #forbiddenError: CustomError = ForbiddenError;
  #readAction: unknown = 'read';

  /**
   * Create an Enforcer, which is used to enforce an Oso policy in an app.
   *
   * @param policy The `Policy` instance to enforce.
   * @param options Optional configuration parameters for this Enforcer.
   */
  constructor(policy: Policy, options: EnforcerOptions<Action> = {}) {
    this.policy = policy;

    if (options.notFoundError) this.#notFoundError = options.notFoundError;
    if (options.forbiddenError) this.#forbiddenError = options.forbiddenError;
    if (options.readAction) this.#readAction = options.readAction;
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
    if (await this.policy.queryRuleOnce('allow', actor, action, resource)) {
      return;
    }

    let isNotFound = false;
    if (action === this.#readAction) {
      isNotFound = true;
    } else if (options.checkRead) {
      const canRead = await this.policy.queryRuleOnce(
        'allow',
        actor,
        this.#readAction,
        resource
      );
      if (!canRead) {
        isNotFound = true;
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
  ): Promise<Array<Action | '*'>> {
    const results = this.policy.queryRule(
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
          return ['*'];
        }
      }
      actions.add(action);
    }
    return Array.from(actions);
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
    const isAllowed = await this.policy.queryRuleOnce(
      'allow_request',
      actor,
      request
    );
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
    const isAllowed = await this.policy.queryRuleOnce(
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
  ): Promise<Array<Field | '*'>> {
    const results = this.policy.queryRule(
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
          return ['*'];
        }
      }
      fields.add(field);
    }
    return Array.from(fields);
  }
}
