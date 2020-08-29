const { Actor, Lab, Order, oso, Test } = require('./inheritanceExternal');

const files = [
  '../01-polar.polar',
  '../02-nested-rule.polar',
  '../03-specializer.polar',
  '../04-one-specializer.polar',
];

describe('inheritance', () => {
  const patient = 'Bob';
  const medStaff = new Actor('medical_staff', [patient]);
  const medStaffBadPatient = new Actor('medical_staff', ['Not Bob']);
  const regStaff = new Actor('reg_staff', [patient]);
  const order = new Order(patient);
  const lab = new Lab(patient);
  const diagnostic = new Test(patient);

  async function loadFile(example) {
    oso.clear();
    await oso.loadFile(example);
    return oso;
  }

  for (const file of files) {
    describe(file.slice(3), () => {
      test('allows medical staff', async () => {
        const oso = await loadFile(file);
        expect(await oso.isAllowed(medStaff, 'read', order)).toBe(true);
        expect(await oso.isAllowed(medStaff, 'read', lab)).toBe(true);
        expect(await oso.isAllowed(medStaff, 'read', diagnostic)).toBe(true);
      });

      test('denies for mismatched patient', async () => {
        await loadFile(file);
        expect(await oso.isAllowed(medStaffBadPatient, 'read', order)).toBe(
          false
        );
        expect(await oso.isAllowed(medStaffBadPatient, 'read', lab)).toBe(
          false
        );
        expect(
          await oso.isAllowed(medStaffBadPatient, 'read', diagnostic)
        ).toBe(false);
      });

      test('denies for regular staff', async () => {
        await loadFile(file);
        expect(await oso.isAllowed(regStaff, 'read', order)).toBe(false);
        expect(await oso.isAllowed(regStaff, 'read', lab)).toBe(false);
        expect(await oso.isAllowed(regStaff, 'read', diagnostic)).toBe(false);
      });
    });
  }
});
