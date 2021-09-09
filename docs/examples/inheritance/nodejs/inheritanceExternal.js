const { Oso } = require('oso');

const oso = new Oso();

class User {
  constructor(role, treated) {
    this.role = role;
    this._treated = treated;
  }

  treated(patient) {
    return this._treated.includes(patient);
  }
}

oso.registerClass(User);

// start-patient-data
class PatientData {
  constructor(patient) {
    this.patient = patient;
  }
}
oso.registerClass(PatientData);

class Lab extends PatientData { }
oso.registerClass(Lab);

class Order extends PatientData { }
oso.registerClass(Order);

class Test extends PatientData { }
oso.registerClass(Test);
// end-patient-data

module.exports = { User, Lab, Order, oso, PatientData, Test };
