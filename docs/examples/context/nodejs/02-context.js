const { Oso } = require('oso');

const oso = new Oso();

class Env {
  static var(variable) {
    return process.env[variable];
  }
}

oso.registerClass(Env);

module.exports = { oso };
