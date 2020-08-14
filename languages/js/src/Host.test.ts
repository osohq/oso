import { Polar as FfiPolar } from './polar_wasm_api';
import { Host } from './Host';
import { pred } from '../test/helpers';
import { Actor, User, Widget } from '../test/classes';
import { Variable } from './Variable';

describe('conversions between JS + Polar values', () => {
  test('converts Polar values into JS values', () => {
    const h = new Host(new FfiPolar());
    const int = 1;
    const float = 3.14159;
    const str = '2';
    const bool = true;
    const list = [int, str, bool];
    const set = new Set([Math.PI, float, Infinity, NaN]);
    const map = new Map([[str, set]]);
    const obj = { [str]: map };
    const value = {
      a: list,
      b: obj,
      c: pred('a', new Actor('b'), new User('c'), new Widget(str)),
      d: new Variable('x'),
    };
    expect(h.toJs(h.toPolarTerm(value))).toStrictEqual(value);
  });
});
