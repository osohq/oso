const assert = require('assert');
const { initOso } = require('./oso');

async function osoInits() {
  const oso = await initOso();
  assert.ok(oso);
}

async function runTests() {
  await osoInits();
  console.log('passed');
}

(async () => {
  await runTests();
})();
