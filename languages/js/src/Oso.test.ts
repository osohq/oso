import { Oso } from './Oso';

class User {
  readonly name: string;
  special: boolean;

  constructor(name: string) {
    this.name = name;
    this.special = false;
  }
}

test('Oso#registerClass', () => {
  const oso = new Oso();
  oso.registerClass(User);
  oso.loadStr('allow(u: User{}, 1, 2) if u.name = "alice";');
  const allowed = oso.isAllowed(new User('alice'), 1, 2);
  expect(allowed).toBe(true);
});
