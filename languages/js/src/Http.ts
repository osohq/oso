export class Http {
  readonly hostname: string;
  readonly path: string;
  readonly query: { [key: string]: string };

  constructor(
    hostname: string,
    path: string,
    query: { [key: string]: string }
  ) {
    this.hostname = hostname;
    this.path = path;
    this.query = query;
  }
}
