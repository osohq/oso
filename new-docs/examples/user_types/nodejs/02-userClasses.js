// classes-start
const { Oso } = require('oso');

const oso = new Oso();

class Customer {
  constructor(id) {
    this.id = id;
  }
}

oso.registerClass(Customer);

// internal-start
class InternalUser {
  // ...

  role() {
    return db.query('SELECT role FROM internal_roles WHERE id = ?', this.id);
  }
}
// internal-end

oso.registerClass(InternalUser);

// account-start
class AccountManager extends InternalUser {
  customerAccounts() {
    return db.query(
      'SELECT id FROM customer_accounts WHERE manager_id = ?',
      this.id
    );
  }
}
// account-end

// generate-start
function userFromId(id) {
  const userType = db.query('SELECT type FROM users WHERE id = ?', id);
  if (userType === 'internal') {
    const actor = new InternalUser(id);
    if (actor.role() === 'account_manager') {
      return new AccountManager(id);
    } else {
      return actor;
    }
  } else if (userType === 'customer') {
    return new Customer(id);
  }
}
// generate-end

module.exports = { oso };
