// classes-start
const { Oso } = require('oso');

const oso = new Oso();

class Customer {
  constructor(id) {
    this.id = id;
  }
}

oso.registerClass(Customer);

class InternalUser {
  constructor(id) {
    this.id = id;
  }
}

oso.registerClass(InternalUser);
// classes-end

// app-start
async function customerDashboardHandler(request) {
  const actor = userFromId(request.id);
  return oso.isAllowed(actor, 'view', 'customer_dashboard');
}

function userFromId(id) {
  const userType = db.query('SELECT type FROM users WHERE id = ?', id);
  if (userType === 'internal') {
    return new InternalUser(id);
  } else if (userType === 'customer') {
    return new Customer(id);
  }
}
// app-end
