// docs: begin-a1
// We're using TypeORM in this example, but you can use any ORM with data filtering.
import { Relation, Oso, ForbiddenError, NotFoundError } from "oso";
import {
  createConnection,
  In,
  Not,
  Entity,
  Column,
  PrimaryColumn,
  PrimaryGeneratedColumn,
} from "typeorm";
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
// This function applies a filter to an existing query.
const constrain = (query, filter) => {
  switch (filter.kind) {
    case "Eq": query[filter.field] = filter.value; break;
    case "Neq": query[filter.field] = Not(filter.value); break;
    case "In": query[filter.field] = In(filter.value); break;
    default:
      throw new Error(`Unknown filter kind: ${filter.kind}`);
  }

  return query;
};

// Create a query from a list of filters
const buildQuery = filters => {
  // TypeORM dislikes empty queries, so give it this instead.
  if (!filters.length) return { id: Not(null) };
  return filters.reduce(constrain, {});
};

// Combine two queries into one
const lift = x => x instanceof Array ? x : [x];
const combineQuery = (a, b) => lift(a).concat(lift(b));

createConnection({
  type: 'sqlite',
  database: ':memory:',
  entities: [User, Repository, RepoRole],
  synchronize: true,
}).then(async connection => {

  // Produce an exec_query function for a class
  const execFromRepo = repo => q =>
    connection.getRepository(repo).find({ where: q });

  const oso = new Oso();

  // The build and combine query implementations are shared in this case,
  // so register them as defaults.
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
  await roles.save({
    user_id: 'leina',
    repo_id: 'oso',
    name: 'contributor'
  });
  await roles.save({
    user_id: 'leina',
    repo_id: 'demo',
    name: 'maintainer'
  });

  // for sorting results
  const compare = (a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0;

  repos.findByIds(['oso', 'demo']).then(repos =>
    users.findOne({ id: 'leina' }).then(leina =>
      oso.authorizedResources(leina, 'read', Repository).then(result =>
        assert.deepEqual(result.sort(compare), repos.sort(compare)))));
});
// docs: end-a3
