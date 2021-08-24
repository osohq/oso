import { ForbiddenError, NotFoundError } from './errors';
import { Policy } from './Oso';

type EnforcerOptions<Action> = {
  getError?: (isNotFound: boolean) => Error;
  readAction?: Action;
};

function defaultGetError(isNotFound: boolean) {
  if (isNotFound) return new NotFoundError();
  return new ForbiddenError();
}
export class Enforcer<
  Actor = unknown,
  Action = String,
  Resource = unknown,
  Field = String,
  Request = unknown
> {
  policy: Policy;
  #getError: (isNotFound: boolean) => Error = defaultGetError;
  readAction: any = 'read';

  constructor(policy: Policy, options: EnforcerOptions<Action> = {}) {
    this.policy = policy;

    if (options.getError) this.#getError = options.getError;
    if (options.readAction) this.readAction = options.readAction;
  }

  async authorize(
    actor: Actor,
    action: Action,
    resource: Resource,
    checkRead: boolean = true
  ): Promise<void> {
    if (!(await this.policy.queryRuleOnce('allow', actor, action, resource))) {
      let isNotFound = false;
      if (action == this.readAction) {
        isNotFound = true;
      } else if (checkRead) {
        if (
          !(await this.policy.queryRuleOnce(
            'allow',
            actor,
            this.readAction,
            resource
          ))
        ) {
          isNotFound = true;
        }
      }
      throw this.#getError(isNotFound);
    }
  }

  async authorizedActions(
    actor: Actor,
    resource: Resource,
    allowWildcard: boolean = false
  ): Promise<Array<Action>> {
    return [];
  }

  async authorizeRequest(actor: Actor, request: Request): Promise<void> {
    if (!(await this.policy.queryRuleOnce('allow_request', actor, request))) {
      throw this.#getError(false);
    }
  }

  async authorizeField(
    actor: Actor,
    action: Action,
    resource: Resource,
    field: Field
  ): Promise<void> {
    if (
      !(await this.policy.queryRuleOnce(
        'allow_field',
        actor,
        action,
        resource,
        field
      ))
    ) {
      throw this.#getError(false);
    }
  }

  async authorizedFields(
    actor: Actor,
    action: Action,
    resource: Resource,
    allowWildcard: boolean = false
  ): Promise<Array<Field>> {
    return [];
  }
}
