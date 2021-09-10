class Repository {
  constructor(name) {
    this.name = name;
  }
}

const repos_db = {
  gmail: new Repository('gmail')
};

// docs: start
class Role {
  constructor(name, repository) {
    this.name = name;
    this.repository = repository;
  }
}

class User {
  constructor(roles) {
    this.roles = roles;
  }
}

const users_db = {
  larry: new User([new Role('admin', repos_db['gmail'])])
};
// docs: end

module.exports = {
  Repository: Repository,
  User: User
};
