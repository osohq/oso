// docs: begin-b1
import { Relation, Oso, ForbiddenError, NotFoundError } from "oso";
import { createConnection, In, Not, Entity, PrimaryGeneratedColumn, Column, PrimaryColumn, JoinColumn, ManyToOne } from "typeorm";
import { readFileSync } from "fs";
import * as assert from 'assert';
import { typeOrmAdapter } from 'oso/dist/src/typeOrmAdapter';

@Entity()
class Repository {
  @PrimaryColumn()
  id: string;
  @Column()
  org_id: string;
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

@Entity()
class Organization {
  @PrimaryColumn()
  id: string;
}

@Entity()
class OrgRole {
  @PrimaryGeneratedColumn()
  id: number;
  @Column()
  name: string;
  @Column()
  user_id: string;
  @Column()
  org_id: string;
}

// docs: end-b1

// docs: begin-b2
async function test() {

  const connection = await createConnection({
    type: 'sqlite',
    database: ':memory:',
    entities: [User, Repository, RepoRole, Organization, OrgRole],
    synchronize: true,
    logging: false,
  });

  const oso = new Oso();
  oso.setDataFilteringAdapter(typeOrmAdapter(connection));

  oso.registerClass(Repository, {
    fields: {
      id: String,
      organization: new Relation("one", "Organization", "org_id", "id"),
    }
  });

  oso.registerClass(Organization, {
    fields: {
      id: String,
      repos: new Relation("many", "Repository", "id", "org_id"),
    }
  });

  oso.registerClass(User, {
    fields: {
      id: String,
      repo_roles: new Relation("many", "RepoRole", "id", "user_id"),
      org_roles: new Relation("many", "OrgRole", "id", "user_id")
    }
  });

  oso.registerClass(RepoRole, {
    fields: {
      id: Number,
      user: new Relation("one", "User", "user_id", "id"),
      repo: new Relation("one", "Repository", "repo_id", "id")
    }
  });

  oso.registerClass(OrgRole, {
    fields: {
      id: Number,
      user: new Relation("one", "User", "user_id", "id"),
      organization: new Relation("one", "Organization", "org_id", "id")
    }
  });
  // docs: end-b2

  // docs: begin-b3
  oso.loadFiles(["policy_b.polar"]);
  const orgs = connection.getRepository(Organization),
    users = connection.getRepository(User),
    repos = connection.getRepository(Repository),
    roles = connection.getRepository(OrgRole);

  await orgs.save({ id: 'osohq' });
  await orgs.save({ id: 'apple' });
  await repos.save({ id: 'ios', org_id: 'apple' });
  await repos.save({ id: 'oso', org_id: 'osohq' });
  await repos.save({ id: 'demo', org_id: 'osohq' });
  await users.save({ id: 'leina' });
  await users.save({ id: 'steve' });
  await roles.save({
    user_id: 'leina',
    org_id: 'osohq',
    name: 'owner'
  });

  // for sorting results
  const compare = (a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0;

  repos.findByIds(['oso', 'demo']).then(repos =>
    users.findOne({ id: 'leina' }).then(leina =>
      oso.authorizedResources(leina, 'read', Repository).then(result =>
        assert.deepEqual(result.sort(compare), repos.sort(compare)))));
}
test()
// docs: end-b3
