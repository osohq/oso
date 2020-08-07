export class PathMapper {
  #pattern: RegExp;

  constructor(_template: string) {
    // const captureGroup = /({([^}]+)})/;
    this.#pattern = /./;
  }
}
