import { Oso } from './Oso';
import { map } from '../test/helpers';

test('#isAllowed', () => {
  const o = new Oso();
  o.loadStr('allow(1, 2, 3);');
  expect(o.isAllowed(1, 2, 3)).toBe(true);
  expect(o.isAllowed(3, 2, 1)).toBe(false);
});

describe('Equality function used for unification', () => {
  test('defaults to loose equality (==)', () => {
    const o = new Oso();
    o.registerConstant('undefined', undefined);
    o.registerConstant('null', null);
    expect(Array.from(o.query('undefined = null'))).toStrictEqual([map()]);
  });

  test('can be overridden with a custom equality function', () => {
    const o = new Oso({ equalityFn: (x, y) => x === y });
    o.registerConstant('undefined', undefined);
    o.registerConstant('null', null);
    expect(Array.from(o.query('undefined = null'))).toStrictEqual([]);
  });
});
