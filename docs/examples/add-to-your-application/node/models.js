class Repository {
  constructor(name) {
    this.name = name;
  }
}

Repository.getByName = name => {
  return repos_db[name];
};

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

User.getCurrentUser = () => {
  return users_db['larry'];
};

const users_db = {
  larry: new User([new Role('admin', repos_db['gmail'])])
};
// docs: end

module.exports = {
  Repository: Repository,
  User: User
};
