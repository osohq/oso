import { Polar as FfiPolar } from './polar_wasm_api';
import { Host } from './Host';
import { pred } from '../test/helpers';
import { Actor, User, Widget } from '../test/classes';
import { Variable } from './Variable';

describe('conversions between JS + Polar values', () => {
  test('converts Polar values into JS values', async () => {
    const h = new Host(new FfiPolar(), (x, y) => x == y);
    const int = 1;
    const float = Math.PI;
    const str = '2';
    const bool = true;
    const list = [int, str, bool];
    const set = new Set([Math, float, Infinity, NaN, undefined, null]);
    const map = new Map([[str, set]]);
    const obj = { [str]: map };
    const promises = {
      resolved: Promise.resolve(int),
      pending: Promise.reject(str).catch(() => {}),
      constructor: Promise.prototype,
    };
    const functions = {
      arrow: () => () => 1,
      expr: function () {
        return function () {
          return 2;
        };
      },
    };
    const value = {
      a: list,
      b: obj,
      c: pred('a', new Actor('b'), new User('c'), new Widget(str)),
      d: new Variable('x'),
      e: promises,
      f: functions,
    };
    expect(await h.toJs(h.toPolar(value))).toStrictEqual(value);
  });
});
