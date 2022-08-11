export class Org {
  readonly name: string;

  constructor(name: string) {
    this.name = name;
  }
}

export class Repo {
  readonly name: string;
  readonly org: Org;

  constructor(name: string, org: Org) {
    this.name = name;
    this.org = org;
  }
}

export class Issue {
  readonly name: string;
  readonly repo: Repo;

  constructor(name: string, repo: Repo) {
    this.name = name;
    this.repo = repo;
  }
}

export class Role {
  readonly name: string;
  readonly resource: Org | Repo;

  constructor(name: string, resource: Org | Repo) {
    this.name = name;
    this.resource = resource;
  }
}

export class User {
  readonly name: string;
  readonly roles: Role[];

  constructor(name: string, roles: Role[]) {
    this.name = name;
    this.roles = roles;
  }
}
