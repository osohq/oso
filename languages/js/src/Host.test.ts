import { Polar as FfiPolar } from './polar_wasm_api';
import { Expression } from './Expression';
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
    const obj = { [str]: bool };
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
    const value = [
      list,
      obj,
      map,
      pred('a', new Actor('b'), new User('c'), new Widget(str)),
      new Variable('x'),
      promises,
      functions,
      new Expression('Eq', [1, 1]),
    ];
    expect(await h.toJs(h.toPolar(value))).toStrictEqual(value);
  });
});
