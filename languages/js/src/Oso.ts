import { Polar } from './Polar';
import { Http } from './Http';
import { PathMapper } from './PathMapper';
import type { OsoOptions } from './types';

export class Oso extends Polar {
  constructor(opts: OsoOptions = {}) {
    super(opts);
    this.registerClass(Http);
    this.registerClass(PathMapper);
  }

  isAllowed(actor: unknown, action: unknown, resource: unknown): boolean {
    const results = this.queryRule('allow', actor, action, resource);
    const { done } = results.next();
    results.return();
    return !done;
  }
}
