import { Oso } from './Oso';
import { Relation, Field } from './dataFiltering';
import 'reflect-metadata';
import {
  OneToMany,
  ManyToOne,
  PrimaryGeneratedColumn,
  Entity,
  PrimaryColumn,
  Column,
  createConnection,
} from 'typeorm';

@Entity()
class Bar {
  @PrimaryColumn()
  id!: string;
  @Column()
  isCool!: boolean;
  @Column()
  isStillCool!: boolean;
  @OneToMany(() => Foo, foo => foo.bar)
  foos!: Foo[];
}

@Entity()
class Foo {
  @PrimaryColumn()
  id!: string;
  @Column()
  barId!: string;
  @Column()
  isFooey!: boolean;
  @ManyToOne(() => Bar, bar => bar.foos)
  bar!: Bar;
  @OneToMany(() => Log, log => log.foo)
  logs!: Log[];
}

@Entity()
class Log {
  @PrimaryColumn()
  id!: string;
  @Column()
  fooId!: string;
  @Column()
  data!: string;
  @ManyToOne(() => Foo, foo => foo.logs)
  foo!: Foo;
}

@Entity()
class Num {
  @PrimaryColumn()
  fooId!: string;
  @PrimaryColumn()
  number!: number;
}

@Entity()
export class Org {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  name!: string;
  @Column()
  base_repo_role!: string;
  @Column()
  billing_address!: string;
  @OneToMany(() => Repo, repo => repo.org)
  repositories!: Repo[];
  @OneToMany(() => OrgRole, org_role => org_role.org)
  roles!: OrgRole[];
}

@Entity()
export class Repo {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  name!: string;
  @Column()
  org_id!: number;
  @ManyToOne(() => Org, org => org.repositories)
  org!: Org;
  @OneToMany(() => Issue, issue => issue.repo)
  issues!: Issue[];
  @OneToMany(() => RepoRole, repo_role => repo_role.repo)
  roles!: RepoRole[];
}

@Entity()
export class Issue {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  title!: string;
  @Column()
  repo_id!: number;
  @ManyToOne(() => Repo, repo => repo.issues)
  repo!: Repo;
}

@Entity()
export class User {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  email!: string;
  @OneToMany(() => RepoRole, repo_role => repo_role.user)
  repo_roles!: RepoRole[];
  @OneToMany(() => OrgRole, org_role => org_role.user)
  org_roles!: OrgRole[];
}

@Entity()
export class RepoRole {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  name!: string;
  @Column()
  repo_id!: number;
  @Column()
  user_id!: number;
  @ManyToOne(() => User, user => user.repo_roles, { eager: true })
  user!: User;
  @ManyToOne(() => Repo, repo => repo.roles, { eager: true })
  repo!: Repo;
}

@Entity()
export class OrgRole {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  name!: string;
  @Column()
  org_id!: number;
  @Column()
  user_id!: number;
  @ManyToOne(() => Org, org => org.roles, { eager: true })
  org!: Org;
  @ManyToOne(() => User, user => user.org_roles, { eager: true })
  user!: User;
}

let i = 0;
const gensym = (tag?: any) => `_${tag}_${i++}`;

async function fixtures() {
  const connection = await createConnection({
    type: 'sqlite',
    database: `:memory:`,
    entities: [Foo, Bar, Log, Num, Org, Repo, User, OrgRole, RepoRole, Issue],
    synchronize: true,
    logging: false,
    name: gensym(),
  });

  const bars = connection.getRepository(Bar);
  const foos = connection.getRepository(Foo);
  const logs = connection.getRepository(Log);
  const nums = connection.getRepository(Num);

  const users = connection.getRepository(User);
  const repos = connection.getRepository(Repo);
  const orgs = connection.getRepository(Org);
  const issues = connection.getRepository(Issue);
  const repoRoles = connection.getRepository(RepoRole);
  const orgRoles = connection.getRepository(OrgRole);

  async function mkBar(id: string, cool: boolean, stillCool: boolean) {
    const bar = new Bar();
    bar.id = id;
    bar.isCool = cool;
    bar.isStillCool = stillCool;
    await bars.save(bar);
    return bar;
  }

  async function mkFoo(id: string, barId: string, fooey: boolean) {
    const foo = new Foo();
    foo.id = id;
    foo.barId = barId;
    foo.isFooey = fooey;
    await foos.save(foo);
    return foo;
  }

  async function mkNum(number: number, fooId: string) {
    const num = new Num();
    num.fooId = fooId;
    num.number = number;
    await nums.save(num);
    return num;
  }

  const helloBar = await mkBar('hello', true, true);
  const byeBar = await mkBar('goodbye', true, false);

  const aFoo = await mkFoo('one', 'hello', false);
  const anotherFoo = await mkFoo('another', 'hello', true);
  const thirdFoo = await mkFoo('next', 'goodbye', true);

  const aLog = await logs.findOneOrFail(
    await logs.save({ id: 'a', fooId: 'one', data: 'hello' })
  );
  const anotherLog = await logs.findOneOrFail(
    await logs.save({ id: 'b', fooId: 'another', data: 'world' })
  );
  const thirdLog = await logs.findOneOrFail(
    await logs.save({ id: 'c', fooId: 'next', data: 'steve' })
  );

  for (let i of [0, 1, 2]) await mkNum(i, 'one');
  for (let i of [0, 1]) await mkNum(i, 'another');
  for (let i of [0]) await mkNum(i, 'next');

  const oso = new Oso();

  const fromRepo = (repo: any, name: string) => {
    const constrain = (query: any, c: any) => {
      let clause,
        rhs,
        sym = gensym(c.field),
        param: any = {};

      if (c.field === undefined) {
        c.field = 'id';
        c.value = c.kind == 'In' ? c.value.map((x: any) => x.id) : c.value.id;
      }

      if (c.value instanceof Field) {
        rhs = `${name}.${c.value.field}`;
      } else {
        rhs = c.kind == 'In' ? `(:...${sym})` : `:${sym}`;
        param[sym] = c.value;
      }

      if (c.kind === 'Eq') clause = `${name}.${c.field} = ${rhs}`;
      else if (c.kind === 'Neq') clause = `${name}.${c.field} <> ${rhs}`;
      else if (c.kind === 'In') clause = `${name}.${c.field} IN ${rhs}`;
      else throw new Error(`Unknown constraint kind: ${c.kind}`);

      return query.andWhere(clause, param);
    };

    return (constraints: any) =>
      constraints.reduce(constrain, repo.createQueryBuilder(name));
  };

  const execQuery = (q: any) => q.getMany();
  const combineQuery = (a: any, b: any) => {
    // this is kind of bad but typeorm doesn't give you a lot of tools
    // for working with queries :(
    const whereClause = (sql: string) => /WHERE (.*)$/.exec(sql)![1];
    a = a.orWhere(whereClause(b.getQuery()), b.getParameters());
    return a.where(`(${whereClause(a.getQuery())})`, a.getParameters());
  };

  // set global exec/combine query functions
  oso.setDataFilteringQueryDefaults({
    execQuery: execQuery,
    combineQuery: combineQuery,
  });

  oso.registerClass(User, {
    buildQuery: fromRepo(users, 'user'),
    types: {
      id: Number,
      email: String,
      repo_roles: new Relation('many', 'RepoRole', 'id', 'user_id'),
      org_roles: new Relation('many', 'OrgRole', 'id', 'user_id'),
    },
  });

  oso.registerClass(Repo, {
    buildQuery: fromRepo(repos, 'repo'),
    types: {
      id: Number,
      name: String,
      org_id: Number,
      org: new Relation('one', 'Org', 'org_id', 'id'),
      roles: new Relation('many', 'RepoRole', 'id', 'repo_id'),
      issues: new Relation('many', 'Issue', 'id', 'repo_id'),
    },
  });

  oso.registerClass(Org, {
    buildQuery: fromRepo(orgs, 'org'),
    types: {
      id: Number,
      name: String,
      billing_address: String,
      base_repo_role: String,
      repos: new Relation('many', 'Repo', 'id', 'org_id'),
      roles: new Relation('many', 'OrgRole', 'id', 'org_id'),
    },
  });

  oso.registerClass(Issue, {
    buildQuery: fromRepo(issues, 'issue'),
    types: {
      id: Number,
      title: String,
      repo_id: Number,
      repo: new Relation('one', 'Repo', 'repo_id', 'id'),
    },
  });

  oso.registerClass(RepoRole, {
    buildQuery: fromRepo(repoRoles, 'repo_role'),
    types: {
      id: Number,
      role: String,
      repo_id: Number,
      user_id: Number,
      user: new Relation('one', 'User', 'user_id', 'id'),
      repo: new Relation('one', 'Repo', 'repo_id', 'id'),
    },
  });

  oso.registerClass(OrgRole, {
    buildQuery: fromRepo(orgRoles, 'org_role'),
    types: {
      id: Number,
      role: String,
      org_id: Number,
      user_id: Number,
      user: new Relation('one', 'User', 'user_id', 'id'),
      org: new Relation('one', 'Org', 'org_id', 'id'),
    },
  });

  oso.registerClass(Bar, {
    buildQuery: fromRepo(bars, 'bar'),
    types: {
      id: String,
      isCool: Boolean,
      isStillCool: Boolean,
      foos: new Relation('many', 'Foo', 'id', 'barId'),
    },
  });

  oso.registerClass(Foo, {
    buildQuery: fromRepo(foos, 'foo'),
    types: {
      id: String,
      barId: String,
      isFooey: Boolean,
      bar: new Relation('one', 'Bar', 'barId', 'id'),
      logs: new Relation('many', 'Log', 'id', 'fooId'),
      numbers: new Relation('many', 'Num', 'id', 'fooId'),
    },
  });

  oso.registerClass(Log, {
    buildQuery: fromRepo(logs, 'log'),
    types: {
      id: String,
      fooId: String,
      data: String,
      foo: new Relation('one', 'Foo', 'fooId', 'id'),
    },
  });

  oso.registerClass(Num, {
    buildQuery: fromRepo(nums, 'num'),
    types: {
      number: Number,
      fooId: String,
      foo: new Relation('one', 'Foo', 'fooId', 'id'),
    },
  });
  const apple = await orgs.findOneOrFail(
      await orgs.save({
        name: 'apple',
        billing_address: 'cupertino,  CA',
        base_repo_role: 'reader',
      })
    ),
    osohq = await orgs.findOneOrFail(
      await orgs.save({
        name: 'osohq',
        billing_address: 'new york, NY',
        base_repo_role: 'reader',
      })
    ),
    tiktok = await orgs.findOneOrFail(
      await orgs.save({
        name: 'tiktok',
        billing_address: 'beijing, CN',
        base_repo_role: 'reader',
      })
    );

  const pol = await repos.findOneOrFail(
      await repos.save({ name: 'pol', org_id: osohq.id })
    ),
    ios = await repos.findOneOrFail(
      await repos.save({ name: 'ios', org_id: apple.id })
    ),
    app = await repos.findOneOrFail(
      await repos.save({ name: 'app', org_id: tiktok.id })
    );

  const bug = await issues.findOneOrFail(
      await issues.save({ title: 'bug', repo_id: pol.id })
    ),
    lag = await issues.findOneOrFail(
      await issues.save({ title: 'lag', repo_id: ios.id })
    );

  const steve = await users.findOneOrFail(
      await users.save({ email: 'steve@osohq.com' })
    ),
    leina = await users.findOneOrFail(
      await users.save({ email: 'leina@osohq.com' })
    ),
    gabe = await users.findOneOrFail(
      await users.save({ email: 'gabe@osohq.com' })
    ),
    gwen = await users.findOneOrFail(
      await users.save({ email: 'gwen@osohq.com' })
    );

  await orgRoles.save({ name: 'owner', org_id: osohq.id, user_id: leina.id });
  await orgRoles.save({ name: 'member', org_id: tiktok.id, user_id: gabe.id });

  await repoRoles.save({ name: 'writer', repo_id: ios.id, user_id: steve.id });
  await repoRoles.save({ name: 'reader', repo_id: app.id, user_id: gwen.id });

  const checkAuthz = async (
    actor: any,
    action: string,
    resource: any,
    expected: any[]
  ) => {
    for (let x of expected)
      expect(await oso.isAllowed(actor, action, x)).toBe(true);
    const actual = await oso.authorizedResources(actor, action, resource);

    expect(actual).toHaveLength(expected.length);
    expect(actual).toEqual(expect.arrayContaining(expected));
  };

  return {
    oso,
    aFoo,
    anotherFoo,
    thirdFoo,
    aLog,
    anotherLog,
    thirdLog,
    helloBar,
    byeBar,
    checkAuthz,
    lag,
    bug,
    apple,
    pol,
    ios,
    osohq,
    tiktok,
    app,
    steve,
    gwen,
    gabe,
    leina,
  };
}

describe('Data filtering using typeorm/sqlite', () => {
  test('dictionary specializers', async () => {
    const { oso, checkAuthz, aFoo, aLog } = await fixtures();
    oso.loadStr(`
      allow(foo: Foo, "glub", _: {foo: foo});
      allow(foo: Foo, "bluh", log) if foo = log.foo;`);
    await checkAuthz(aFoo, 'glub', Log, [aLog]);
    await checkAuthz(aFoo, 'bluh', Log, [aLog]);
  });
  test('pattern specializers', async () => {
    const { oso, checkAuthz, aFoo, aLog } = await fixtures();
    oso.loadStr(`
      allow(foo: Foo, "glub", _: Log{foo: foo});
      allow(foo: Foo, "bluh", log: Log) if foo = log.foo;`);
    await checkAuthz(aFoo, 'glub', Log, [aLog]);
    await checkAuthz(aFoo, 'bluh', Log, [aLog]);
  });
  test('relations and operators', async () => {
    const { oso, checkAuthz, aFoo, anotherFoo, thirdFoo } = await fixtures();

    oso.loadStr(`
      allow("steve", "get", resource: Foo) if
          resource.bar = bar and
          bar.isCool = true and
          resource.isFooey = true;
      allow("steve", "patch", foo: Foo) if
        foo in foo.bar.foos;
      allow(num: Integer, "count", foo: Foo) if
        rec in foo.numbers and
        rec.number = num;`);

    await checkAuthz('steve', 'get', Foo, [anotherFoo, thirdFoo]);
    await checkAuthz('steve', 'patch', Foo, [aFoo, anotherFoo, thirdFoo]);

    await checkAuthz(0, 'count', Foo, [aFoo, anotherFoo, thirdFoo]);
    await checkAuthz(1, 'count', Foo, [aFoo, anotherFoo]);
    await checkAuthz(2, 'count', Foo, [aFoo]);
  });

  test('an empty result', async () => {
    const { oso } = await fixtures();
    oso.loadStr('allow("gwen", "put", _: Foo);');
    expect(await oso.authorizedResources('gwen', 'delete', Foo)).toEqual([]);
  });

  test('not equals', async () => {
    const { oso, checkAuthz, byeBar } = await fixtures();
    oso.loadStr(`
      allow("gwen", "get", bar: Bar) if
        bar.isCool != bar.isStillCool;`);
    await checkAuthz('gwen', 'get', Bar, [byeBar]);
  });

  test('returning, modifying and executing a query', async () => {
    const { oso, aFoo, anotherFoo } = await fixtures();
    oso.loadStr(`
      allow("gwen", "put", foo: Foo) if
        rec in foo.numbers and
        rec.number in [1, 2];`);

    const query = await oso.authorizedQuery('gwen', 'put', Foo);

    let result = await query.getMany();
    expect(result).toHaveLength(2);
    expect(result).toEqual(expect.arrayContaining([aFoo, anotherFoo]));

    result = await query.andWhere("id = 'one'").getMany();
    expect(result).toHaveLength(1);
    expect(result).toEqual(expect.arrayContaining([aFoo]));
  });

  test('a gitclub-like policy', async () => {
    const {
      oso,
      osohq,
      apple,
      tiktok,
      checkAuthz,
      gwen,
      lag,
      bug,
      steve,
      gabe,
      leina,
      pol,
      app,
      ios,
    } = await fixtures();
    await oso.loadStr(`
allow(actor, action, resource) if
  has_permission(actor, action, resource);

# Users can see each other.
has_permission(_: User, "read", _: User);

# A User can read their own profile.
has_permission(_: User{id: id}, "read_profile", _:User{id: id});

# Any logged-in user can create a new org.
has_permission(_: User, "create", _: Org);

actor User {}

resource Org {
  roles = ["owner", "member"];
  permissions = [
    "read",
    "create_repos",
    "list_repos",
    "create_role_assignments",
    "list_role_assignments",
    "update_role_assignments",
    "delete_role_assignments",
  ];

  "read" if "member";
  "list_repos" if "member";
  "list_role_assignments" if "member";

  "create_repos" if "owner";
  "create_role_assignments" if "owner";
  "update_role_assignments" if "owner";
  "delete_role_assignments" if "owner";

  "member" if "owner";
}

has_role(user: User, name: String, org: Org) if
    role in user.org_roles and
    role matches { name: name, org: org };

resource Repo {
  roles = ["admin", "writer", "reader"];
  permissions = [
    "read",
    "create_issues",
    "list_issues",
    "create_role_assignments",
    "list_role_assignments",
    "update_role_assignments",
    "delete_role_assignments",
  ];
  relations = { parent: Org };

  "create_role_assignments" if "admin";
  "list_role_assignments" if "admin";
  "update_role_assignments" if "admin";
  "delete_role_assignments" if "admin";

  "create_issues" if "writer";

  "read" if "reader";
  "list_issues" if "reader";

  "admin" if "owner" on "parent";
  "reader" if "member" on "parent";

  "writer" if "admin";
  "reader" if "writer";
}

has_role(user: User, name: String, repo: Repo) if
    role in user.repo_roles and
    role matches { name: name, repo: repo };

has_relation(org: Org, "parent", repo: Repo) if org = repo.org;

resource Issue {
  permissions = ["read"];
  relations = { parent: Repo };

  "read" if "reader" on "parent";
}

has_relation(repo: Repo, "parent", issue: Issue) if issue.repo = repo;
    `);

    await checkAuthz(steve, 'create_issues', Repo, [ios]);
    await checkAuthz(steve, 'read', Issue, [lag]);
    await checkAuthz(gwen, 'read', Repo, [app]);
    await checkAuthz(gwen, 'read', Issue, []);
    await checkAuthz(gwen, 'create_issues', Repo, []);
    await checkAuthz(leina, 'create_issues', Repo, [pol]);
    await checkAuthz(gabe, 'create_issues', Repo, []);
  });

  test('a roles policy', async () => {
    const { oso, checkAuthz, aFoo, anotherFoo, helloBar } = await fixtures();
    oso.loadStr(`
      allow(actor, action, resource) if
        has_permission(actor, action, resource);

      has_role("steve", "owner", bar: Bar) if
        bar.id = "hello";

      actor String {}

      resource Bar {
        roles = [ "owner" ];
        permissions = [ "get" ];

        "get" if "owner";
      }

      resource Foo {
        roles = [ "reader" ];
        permissions = [ "read" ];
        relations = { parent: Bar };

        "read" if "reader";

        "reader" if "owner" on "parent";
      }

      has_relation(bar: Bar, "parent", foo: Foo) if
        bar = foo.bar;
      `);
    await checkAuthz('steve', 'get', Bar, [helloBar]);
    await checkAuthz('steve', 'read', Foo, [aFoo, anotherFoo]);
  });
});
