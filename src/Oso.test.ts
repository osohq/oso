import { Oso } from './Oso';
import { map, query } from '../test/helpers';

test('#isAllowed', async () => {
  const o = new Oso();
  await o.loadStr('allow(1, 2, 3);');
  expect(await o.isAllowed(1, 2, 3)).toBe(true);
  expect(await o.isAllowed(3, 2, 1)).toBe(false);
});

describe('Equality function used for unification', () => {
  test('can be overridden with a custom equality function', async () => {
    const o = new Oso({ equalityFn: () => true });
    expect(
      await query(o, 'new String("lol") = new Integer("wut")')
    ).toStrictEqual([map()]);
  });
});
