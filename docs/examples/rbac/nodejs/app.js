const { Oso } = require('oso');

// docs: begin-types
class Organization {
  constructor(name) {
    this.name = name;
  }
}

class Repository {
  constructor(name, organization) {
    this.name = name;
    this.organization = organization;
  }
}

class Role {
  constructor(name, resource) {
    this.name = name;
    this.resource = resource;
  }
}

class User {
  constructor(name) {
    this.name = name;
    this.roles = new Set();
  }

  assignRoleForResource(name, resource) {
    this.roles.add(new Role(name, resource));
  }
}
// docs: end-types

// docs: begin-setup
const oso = new Oso();

// docs: begin-register
oso.registerClass(Organization);
oso.registerClass(Repository);
oso.registerClass(User);
// docs: end-register

(async function() { // docs: hide
await oso.loadFiles(["main.polar"]);
})(); // docs: hide
// docs: end-setup

module.exports = {
  Organization,
  Repository,
  User,
};
