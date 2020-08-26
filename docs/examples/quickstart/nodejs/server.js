const { createServer } = require('http');
const { inspect } = require('util');
const { Oso } = require('oso');
const { Expense, EXPENSES } = require('./expense');

async function start() {
  const oso = new Oso();
  oso.registerClass(Expense);
  await oso.loadFile('expenses.polar');

  createServer(async function (req, res) {
    const actor = req.headers['user'];
    const action = req.method;
    const [, resourceType, id] = req.url.split('/');
    const resource = EXPENSES[parseInt(id)];

    if (resourceType !== 'expenses' || !resource) {
      res.write('Not Found!');
    } else if (await oso.isAllowed(actor, action, resource)) {
      res.write(inspect(resource));
    } else {
      res.write('Not Authorized!');
    }
    res.end();
  }).listen(5050, () => console.log('running on port 5050'));
}

start();
