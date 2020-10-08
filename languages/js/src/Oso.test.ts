import { Oso } from './Oso';
import { map, query } from '../test/helpers';

test('#isAllowed', async () => {
  const o = new Oso();
  o.loadStr('allow(1, 2, 3);');
  expect(await o.isAllowed(1, 2, 3)).toBe(true);
  expect(await o.isAllowed(3, 2, 1)).toBe(false);
});

describe('Equality function used for unification', () => {
  test('defaults to loose equality (==)', async () => {
    const o = new Oso();
    o.registerConstant(undefined, 'undefined');
    o.registerConstant(null, 'null');
    expect(await query(o, 'undefined = null')).toStrictEqual([map()]);
  });

  test('can be overridden with a custom equality function', async () => {
    const o = new Oso({ equalityFn: (x, y) => x === y });
    o.registerConstant(undefined, 'undefined');
    o.registerConstant(null, 'null');
    expect(await query(o, 'undefined = null')).toStrictEqual([]);
  });
});
