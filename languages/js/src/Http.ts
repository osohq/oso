export class Http {
  readonly hostname: string;
  readonly path: string;
  readonly query: Map<string, string>;

  constructor(hostname: string, path: string, query: Map<string, string>) {
    this.hostname = hostname;
    this.path = path;
    this.query = query;
  }
}
