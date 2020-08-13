import type { obj } from './types';

export class Http {
  readonly hostname: string;
  readonly path: string;
  readonly query: obj;

  constructor(hostname: string, path: string, query: obj) {
    this.hostname = hostname;
    this.path = path;
    this.query = query;
  }
}
