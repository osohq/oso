import { Policy } from './Oso';

export class Enforcer<
  Actor = unknown,
  Action = unknown,
  Resource = unknown,
  Field = String,
  Request = unknown
> {
  policy: Policy;

  constructor(policy: Policy) {
    this.policy = policy;
  }

  async authorize(
    actor: Actor,
    action: Action,
    resource: Resource
  ): Promise<null> {
    return null;
  }

  async authorizedActions(
    actor: Actor,
    resource: Resource,
    allowWildcard: boolean = false
  ): Promise<Array<Action>> {
    return [];
  }

  async authorizeRequest(actor: Actor, request: Request): Promise<null> {
    return null;
  }

  async authorizeField(
    actor: Actor,
    action: Action,
    resource: Resource,
    field: Field
  ): Promise<null> {
    return null;
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
