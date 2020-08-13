import { Oso } from './Oso';

test('#isAllowed', () => {
  const o = new Oso();
  o.loadStr('allow(1, 2, 3);');
  expect(o.isAllowed(1, 2, 3)).toBe(true);
  expect(o.isAllowed(3, 2, 1)).toBe(false);
});
