const express = require('express');

const { initOso } = require('./oso');
const { User, Repository } = require('./models');

const app = express();

app.use(async (req, res, next) => {
  res.locals.oso = await initOso();
  next();
});

// docs: begin-show-route
app.get('/repo/:name', async (req, res) => {
  const repo = Repository.getByName(req.params.name);

  try {
    // docs: begin-authorize
    await res.locals.oso.authorize(User.getCurrentUser(), 'read', repo);
    // docs: end-authorize
    return res.end(`<h1>A Repo</h1><p>Welcome to repo ${repo.name}</p>`);
  } catch (e) {
    console.log(e);
    return res
      .status(404)
      .end(`<h1>Whoops!</h1><p>Repo named ${name} was not found</p>`);
  }
});
// docs: end-show-route

module.exports = {
  app
};
