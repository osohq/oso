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

async function dataFilteringWorks() {
  const { app } = require('./dataFiltering');
  await request(app)
    .get('/repos')
    .expect(200)
    .then(response => {
      console.log(response.text);
      assert(response.text.length >= 3);
    });
}

async function runTests() {
  await osoInits();
  await routeWorks();
  await dataFilteringWorks();
}

(async () => {
  await runTests();
})();
