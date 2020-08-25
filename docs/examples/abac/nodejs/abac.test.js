const { oso, Expense, User } = require('./01-simple');

const EXPENSES_DEFAULT = {
  submitted_by: 'steve',
  location: 'NYC',
  amount: 50,
  project_id: 2,
};
const sam = new User('sam');

test('01-simple', async () => {
  await oso.loadFile('../01-simple.polar');
  const samEx = new Expense({ ...EXPENSES_DEFAULT, submitted_by: sam.name });
  expect(await oso.isAllowed(sam, 'view', samEx)).toBe(true);
  const steveEx = new Expense({ ...EXPENSES_DEFAULT });
  expect(await oso.isAllowed(sam, 'view', steveEx)).toBe(false);
});

test('02-rbac', async () => {
  await oso.loadFile('../02-rbac.polar');
  await oso.loadStr(
    'role(_: User { name: "sam" }, "admin", _: Project { id: 2 });'
  );
  const proj0Ex = new Expense({ ...EXPENSES_DEFAULT, project_id: 0 });
  expect(await oso.isAllowed(sam, 'view', proj0Ex)).toBe(false);
  const proj2Ex = new Expense({ ...EXPENSES_DEFAULT });
  expect(await oso.isAllowed(sam, 'view', proj2Ex)).toBe(true);
});

test('03-hierarchy', async () => {
  await oso.loadFile('../03-hierarchy.polar');
  const bhavik = new User('bhavik');
  const aliceEx = new Expense({ ...EXPENSES_DEFAULT, submitted_by: 'alice' });
  expect(await oso.isAllowed(bhavik, 'view', aliceEx)).toBe(true);
});
