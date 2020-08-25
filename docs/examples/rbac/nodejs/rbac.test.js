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
  expect(oso.isAllowed(employee, 'submit', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(admin, 'approve', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(employee, 'approve', 'expense')).resolves.toBe(false);
  expect(oso.isAllowed(accountant, 'view', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(greta, 'approve', 'expense')).resolves.toBe(true);
});

test('06-external', async () => {
  const oso = await loadFile('../06-external.polar');
  expect(oso.isAllowed(employee, 'submit', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(employee, 'view', 'expense')).resolves.toBe(false);
  expect(oso.isAllowed(employee, 'approve', 'expense')).resolves.toBe(false);
  expect(oso.isAllowed(accountant, 'submit', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(accountant, 'view', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(accountant, 'approve', 'expense')).resolves.toBe(false);
  expect(oso.isAllowed(admin, 'submit', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(admin, 'view', 'expense')).resolves.toBe(true);
  expect(oso.isAllowed(admin, 'approve', 'expense')).resolves.toBe(true);
});
