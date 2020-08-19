import { Polar } from './Polar';
import { Http } from './Http';
import { PathMapper } from './PathMapper';
import type { Options } from './types';

export class Oso extends Polar {
  constructor(opts: Options = {}) {
    super(opts);
    this.registerClass(Http);
    this.registerClass(PathMapper);
  }

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
