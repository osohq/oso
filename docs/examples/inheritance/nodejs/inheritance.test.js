const { Oso } = require('oso');
const {
  User,
  Lab,
  Order,
  PatientData,
  Test,
} = require('./inheritanceExternal');

const files = [
  '../01-polar.polar',
  '../02-nested-rule.polar',
  '../03-specializer.polar',
  '../04-one-specializer.polar',
];

describe('inheritance', () => {
  const patient = 'Bob';
  const medStaff = new User('medical_staff', [patient]);
  const medStaffBadPatient = new User('medical_staff', ['Not Bob']);
  const regStaff = new User('reg_staff', [patient]);
  const order = new Order(patient);
  const lab = new Lab(patient);
  const diagnostic = new Test(patient);

  async function loadFile(example) {
    const oso = new Oso();
    oso.registerClass(User);
    oso.registerClass(Lab);
    oso.registerClass(Order);
    oso.registerClass(PatientData);
    oso.registerClass(Test);
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
        const oso = await loadFile(file);
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
        const oso = await loadFile(file);
        expect(await oso.isAllowed(regStaff, 'read', order)).toBe(false);
        expect(await oso.isAllowed(regStaff, 'read', lab)).toBe(false);
        expect(await oso.isAllowed(regStaff, 'read', diagnostic)).toBe(false);
      });
    });
  }
});
