// docs: begin-a1
import { Relation, Oso, ForbiddenError, NotFoundError } from "oso";
import { createConnection, In, Not, Entity, PrimaryGeneratedColumn, Column, PrimaryColumn, JoinColumn, ManyToOne } from "typeorm";
import { readFileSync } from "fs";
import * as assert from 'assert';

@Entity()
class Repository {
  @PrimaryColumn()
  id: string;
}

@Entity()
class User {
  @PrimaryColumn()
  id: string;
}

@Entity()
class RepoRole {
  @PrimaryGeneratedColumn()
  id: number;
  @Column()
  name: string;
  @Column()
  repo_id: string;
  @Column()
  user_id: string;
}

// docs: end-a1

// docs: begin-a2
const constrain = (query, filter) => {
  if (filter.field === undefined) {
    filter.field = "id";
    if (filter.kind == "In")
      filter.value = filter.value.map(v => v.id);
    else
      filter.value = filter.value.id;
  }
  switch (filter.kind) {
    case "Eq": query[filter.field] = filter.value; break;
    case "Neq": query[filter.field] = Not(filter.value); break;
    case "In": query[filter.field] = In(filter.value); break;
    default:
      throw new Error(`Unknown filter kind: ${filter.kind}`);
  }

  return query;
};

const buildQuery = filters => {
  if (!filters.length) return { id: Not(null) };
  return filters.reduce(constrain, {});
};

const lift = x => x instanceof Array ? x : [x];
const combineQuery = (a, b) => lift(a).concat(lift(b));

createConnection({
  type: 'sqlite',
  database: ':memory:',
  entities: [User, Repository, RepoRole],
  synchronize: true,
}).then(async connection => {

  const execFromRepo = repo => q =>
    connection.getRepository(repo).find({ where: q });

  const oso = new Oso();

  oso.setDataFilteringQueryDefaults({ combineQuery, buildQuery });

  oso.registerClass(Repository, {
    execQuery: execFromRepo(Repository),
    types: { id: String }
  });

  oso.registerClass(User, {
    execQuery: execFromRepo(User),
    types: {
      id: String,
      repo_roles: new Relation("many", "RepoRole", "id", "user_id")
    }
  });

  oso.registerClass(RepoRole, {
    execQuery: execFromRepo(RepoRole),
    types: {
      id: Number,
      user: new Relation("one", "User", "user_id", "id"),
      repo: new Relation("one", "Repo", "repo_id", "id")
    }
  });

  // docs: end-a2

  // docs: begin-a3
  oso.loadFiles(["policy_a.polar"]);
  const users = connection.getRepository(User),
    repos = connection.getRepository(Repository),
    roles = connection.getRepository(RepoRole);

  await repos.save({ id: 'ios' });
  await repos.save({ id: 'oso' });
  await repos.save({ id: 'demo' });
  await users.save({ id: 'leina' });
  await users.save({ id: 'steve' });
  await roles.save({ user_id: 'leina', repo_id: 'oso', name: 'contributor' }),
    await roles.save({ user_id: 'leina', repo_id: 'demo', name: 'maintainer' });

  const compare = (a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0;

  repos.findByIds(['oso', 'demo']).then(repos =>
    users.findOne({ id: 'leina' }).then(leina =>
      oso.authorizedResources(leina, 'read', Repository).then(result =>
        assert.deepEqual(result.sort(compare),
          repos.sort(compare)))));
});
// docs: end-a3