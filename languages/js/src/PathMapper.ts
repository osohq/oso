import type { obj } from './types';

/**
 * Utility class to map from a forward-slash-delimited path to a dictionary
 * of matched path segments.
 */
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

  /**
   * Apply the templated pattern to a provided string, returning an object of
   * matching capture groups.
   */
  map(str: string): obj {
    return { ...this.#pattern.exec(str)?.groups };
  }
}
