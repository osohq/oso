export class PathMapper {
  #pattern: RegExp;

  constructor(template: string) {
    // TODO(gj): can probably do better than `[^}]` for the inner capture
    // group.
    const captureGroup = /({([^}]+)})/g;
    let temp = template;
    let captures;
    while ((captures = captureGroup.exec(template)) !== null) {
      const [, outer, inner] = captures;
      if (inner === '*') {
        temp = temp.replace(new RegExp(outer, 'g'), '.*');
      } else {
        temp = temp.replace(new RegExp(outer, 'g'), `(?<${inner}>[^/]+)`);
      }
    }
    this.#pattern = new RegExp(`^${temp}$`);
  }

  map(str: string): { [key: string]: any } {
    return { ...this.#pattern.exec(str)?.groups };
  }
}
