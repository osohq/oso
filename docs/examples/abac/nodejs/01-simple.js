const EXPENSES = [
  { amount: 500, submitted_by: 'alice', location: 'NYC', project_id: 2 },
];

// expense-class-start
class Expense {
  // expense-class-end

  constructor({ amount, submitted_by, location, project_id }) {
    this.amount = amount;
    this.submitted_by = submitted_by;
    this.location = location;
    this.project_id = project_id;
  }

  static id(id) {
    if (id < EXPENSES.length) return new Expense({ ...EXPENSES[id] });
    return new Expense();
  }
}

const MANAGERS = {
  cora: ['bhavik'],
  bhavik: ['alice'],
};

// user-class-start
class User {
  constructor(name, location) {
    this.name = name; // user-class-end
    this.location = location || 'NYC';
  }

  *employees() {
    if (MANAGERS[this.name]) {
      for (const name in MANAGERS[this.name]) {
        yield new User(name);
      }
    }
  }
}

class Project {
  constructor(id, teamId) {
    this.id = id;
    this.teamId = teamId;
  }

  static id(id) {
    return new Project(id, 0);
  }
}

class Team {
  constructor(organizationId) {
    this.organizationId = organizationId;
  }

  static id() {
    return new Team(0);
  }
}

class Organization {
  constructor(name) {
    this.name = name;
  }

  static id() {
    return new Organization('ACME');
  }
}

module.exports = {
  Expense,
  Organization,
  Project,
  Team,
  User,
};
