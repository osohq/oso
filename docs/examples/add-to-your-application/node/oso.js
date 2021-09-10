const { Oso } = require('oso');
const { Repository, User } = require('./models');

async function initOso() {
  // Initialize the Oso object. This object is usually
  // used globally throughout an application.
  const oso = new Oso();

  // Tell Oso about the data that you will authorize.
  // These types can be referenced in the policy.
  oso.registerClass(User);
  oso.registerClass(Repository);

  // Load your policy file.
  await oso.loadFiles(['main.polar']);

  return oso;
}

module.exports = {
  initOso: initOso
};
