import { Polar as FfiPolar } from './polar_wasm_api';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import { Dict } from './types';
import { Host } from './Host';
import { pred } from '../test/helpers';
import { BaseActor, User, Widget } from '../test/classes';
import { Variable } from './Variable';

describe('conversions between JS + Polar values', () => {
  test('converts Polar values into JS values', async () => {
    const h = new Host(new FfiPolar(), {
      acceptExpression: true,
      equalityFn: (x, y) => x == y, // eslint-disable-line eqeqeq
    });
    const int = 1;
    const float = Math.PI;
    const str = '2';
    const bool = true;
    const list = [int, str, bool];
    const set = new Set([Math, float, Infinity, NaN, undefined, null]);
    const map = new Map([[str, set]]);
    const obj = { [str]: bool };
    const dict = new Dict({});
    dict[str] = bool;
    const instancePattern = new Pattern({ tag: str, fields: dict });
    const dictPattern = new Pattern({ fields: dict });
    const promises = {
      resolved: Promise.resolve(int),
      pending: Promise.reject(str).catch(() => {}), // eslint-disable-line @typescript-eslint/no-empty-function
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
      dict,
      instancePattern,
      dictPattern,
      list,
      obj,
      map,
      pred('a', new BaseActor('b'), new User('c'), new Widget(str)),
      new Variable('x'),
      promises,
      functions,
      new Expression('Eq', [1, 1]),
    ];
    expect(await h.toJs(h.toPolar(value))).toStrictEqual(value);
  });
});
