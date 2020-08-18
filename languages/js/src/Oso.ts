import { Polar } from './Polar';
import { Http } from './Http';
import { PathMapper } from './PathMapper';

export class Oso extends Polar {
  constructor() {
    super();
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
