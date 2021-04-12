const { Oso } = require('oso');

class User {
  constructor(name, role) {
    this.name = name;
    this.role = role;
  }
}

async function loadFile(example) {
  const oso = new Oso();
  oso.registerClass(User);
  await oso.loadFile(example);
  return oso;
}

test('01-simple', () => {
  expect(loadFile('../01-simple.polar')).resolves.not.toThrow();
});

test('02-simple', () => {
  expect(loadFile('../02-simple.polar')).resolves.not.toThrow();
});

const admin = new User('alice', 'admin');
const accountant = new User('armando', 'accountant');
const employee = new User('eli', 'employee');
const greta = new User('greta');

test('05-external', async () => {
  const oso = await loadFile('../05-external.polar');
  expect(await oso.isAllowed(employee, 'submit', 'expense')).toBe(true);
  expect(await oso.isAllowed(admin, 'approve', 'expense')).toBe(true);
  expect(await oso.isAllowed(employee, 'approve', 'expense')).toBe(false);
  expect(await oso.isAllowed(accountant, 'view', 'expense')).toBe(true);
  expect(await oso.isAllowed(greta, 'approve', 'expense')).toBe(true);
});

test('06-external', async () => {
  const oso = await loadFile('../06-external.polar');
  expect(await oso.isAllowed(employee, 'submit', 'expense')).toBe(true);
  expect(await oso.isAllowed(employee, 'view', 'expense')).toBe(false);
  expect(await oso.isAllowed(employee, 'approve', 'expense')).toBe(false);
  expect(await oso.isAllowed(accountant, 'submit', 'expense')).toBe(true);
  expect(await oso.isAllowed(accountant, 'view', 'expense')).toBe(true);
  expect(await oso.isAllowed(accountant, 'approve', 'expense')).toBe(false);
  expect(await oso.isAllowed(admin, 'submit', 'expense')).toBe(true);
  expect(await oso.isAllowed(admin, 'view', 'expense')).toBe(true);
  expect(await oso.isAllowed(admin, 'approve', 'expense')).toBe(true);
});
