const express = require('express');
const { Oso } = require('oso');
const { User } = require('./models');

const { Sequelize, Model, DataTypes } = require('sequelize');
const sequelize = new Sequelize('sqlite::memory:');

class Repository extends Model {}
Repository.init(
  {
    name: DataTypes.STRING,
    isPublic: DataTypes.BOOLEAN
  },
  { sequelize, modelName: 'repository' }
);

async function initDb() {
  await sequelize.sync();
  await Repository.create({
    name: 'gmail'
  });
  await Repository.create({
    name: 'notgmail'
  });

  return sequelize;
}

// docs: begin-data-filtering
// This is an example implementation for the Sequelize ORM, but you can
// use any ORM with this API.
async function initOso() {
  await initDb();

  function getRepositories(filters) {
    const sequelizeFilters = filters.map(filter => {
      let value = filter.value;
      let field = filter.field;
      if (field === undefined) {
        value = value.name;
        field = 'name';
      }

      if (filter.kind === 'Eq' || filter.kind === 'In') {
        return [field, value];
      } else {
        throw new Error('Unsupported filter type.');
      }
    });

    const where = {};
    for (const filter of sequelizeFilters) {
      where[filter[0]] = filter[1];
    }

    console.log(where);
    return where;
  }

  const oso = new Oso();

  oso.registerClass(User);
  oso.registerClass(Repository, {
    name: 'Repository',
    types: {
      isPublic: Boolean
    },
    buildQuery: getRepositories,
    execQuery: async w => await Repository.findAll({ where: w }),
    combineQuery: (q1, q2) => Object.assign({}, q1, q2)
  });

  await oso.loadFiles(['main.polar']);

  return oso;
}
// docs: end-data-filtering

function getCurrentUser() {
  return new User([
    { name: 'admin', repository: new Repository({ name: 'gmail' }) }
  ]);
}

const app = express();
app.use(async (req, res, next) => {
  const oso = await initOso();
  res.locals.oso = oso;
  next();
});

function serialize(r) {
  return r.toString();
}

// docs: begin-list-route
app.get('/repos', async (req, res) => {
  const repositories = await res.locals.oso.authorizedResources(
    getCurrentUser(),
    'read',
    Repository
  );

  res.end(serialize(repositories));
});
// docs: end-list-route

module.exports = {
  app
};
