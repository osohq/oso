const root: Function = Object.getPrototypeOf(() => {});
export function ancestors(cls: Function): Function[] {
  const ancestors = [cls];
  function next(cls: Function): void {
    try {
      const parent = Object.getPrototypeOf(cls);
      if (parent === root) return;
      ancestors.push(parent);
      next(parent);
    } catch (e) {
      // TODO(gj): should it be a silent failure if something weird's in the
      // prototype chain?
      return;
    }
  }
  next(cls);
  return ancestors;
}
