import { Polar } from './Polar';
import type { Options } from './types';

/** The oso authorization API. */
export class Oso extends Polar {
  constructor(opts: Options = {}) {
    super(opts);
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
    actor: unknown,
    action: unknown,
    resource: unknown
  ): Promise<boolean> {
    const results = this.queryRule('allow', actor, action, resource);
    const { done } = await results.next();
    results.return();
    return !done;
  }
}
