const { Oso } = require('oso');

const oso = new Oso();

(async () => {
  const actor = 'alice@example.com';
  const resource = EXPENSES[1];
  await oso.isAllowed(actor, 'GET', resource);
})();
