const { Oso } = require('oso');

const oso = new Oso();

// context-start
class Env {
  static var(variable) {
    return process.env[variable];
  }
}

oso.registerClass(Env);
// context-end

module.exports = { oso };
