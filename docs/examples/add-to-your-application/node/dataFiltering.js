const { Oso } = require('oso');

const { Sequelize, Model, DataTypes } = require('sequelize');
const sequelize = new Sequelize('sqlite::memory:');

class Repository extends Model {}
Repository.init(
  {
    name: DataTypes.STRING,
    isPublic: DataTypes.BOOLEAN
  },
  { sequelize, modelName: 'user' }
);
