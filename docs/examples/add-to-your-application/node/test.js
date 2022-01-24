const assert = require('assert');
const { initOso } = require('./oso');
const { app } = require('./routes');
const request = require('supertest');

async function osoInits() {
  const oso = await initOso();
  assert.ok(oso);
}

async function routeWorks() {
  await request(app)
    .get('/repo/gmail')
    .expect(200);
}

async function runTests() {
  await osoInits();
  await routeWorks();
}

(async () => {
  await runTests();
})();
