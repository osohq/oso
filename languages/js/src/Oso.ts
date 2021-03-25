import { Polar } from './Polar';
import type { Options } from './types';

function maybeReadOptionsFromEnv(opts: Options = {}): Options {
  if (typeof process === 'undefined') return opts;
  if (opts.log === undefined) opts.log = !!process.env.RUST_LOG;
  if (opts.polarLog === undefined) opts.polarLog = !!process.env.POLAR_LOG;
  if (opts.polarLogStderr === undefined)
    opts.polarLogStderr = process.env.POLAR_LOG === 'now';
  return opts;
}

/** The oso authorization API. */
export class Oso extends Polar {
  constructor(opts: Options = {}) {
    super(maybeReadOptionsFromEnv(opts));
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
