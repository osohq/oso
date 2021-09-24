import { Oso } from './Oso';
import { Field, Relation } from './dataFiltering';
import type { Filter } from './dataFiltering';
import 'reflect-metadata';
import {
  OneToMany,
  ManyToOne,
  PrimaryGeneratedColumn,
  Entity,
  PrimaryColumn,
  Column,
  createConnection,
  Repository,
  SelectQueryBuilder,
  DeepPartial,
} from 'typeorm';
import { Class, obj } from './types';

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
  orgId!: number;
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
  repoId!: number;
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
  repoRoles!: RepoRole[];
  @OneToMany(() => OrgRole, org_role => org_role.user)
  orgRoles!: OrgRole[];
}

@Entity()
export class RepoRole {
  @PrimaryGeneratedColumn()
  id!: number;
  @Column()
  name!: string;
  @Column()
  repoId!: number;
  @Column()
  userId!: number;
  @ManyToOne(() => User, user => user.repoRoles, { eager: true })
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
  orgId!: number;
  @Column()
  userId!: number;
  @ManyToOne(() => Org, org => org.roles, { eager: true })
  org!: Org;
  @ManyToOne(() => User, user => user.orgRoles, { eager: true })
  user!: User;
}

let i = 0;
const gensym = (tag?: string) => `_${tag || 'anon'}_${i++}`;

async function fixtures() {
  const connection = await createConnection({
    type: 'sqlite',
    database: ':memory:',
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

  for (const i of [0, 1, 2]) await mkNum(i, 'one');
  for (const i of [0, 1]) await mkNum(i, 'another');
  for (const i of [0]) await mkNum(i, 'next');

  const oso = new Oso();

  const fromRepo = <T>(repo: Repository<T>, name: string) => {
    const constrain = (
      query: SelectQueryBuilder<T>,
      { field, kind, value }: Filter
    ) => {
      const sym = gensym(field);
      const param: obj = {};

      if (field === undefined) {
        field = 'id';
        value =
          kind === 'In' ? (value as obj[]).map(x => x.id) : (value as obj).id; // eslint-disable-line @typescript-eslint/no-explicit-any
      }

      let rhs: string;
      if (value instanceof Field) {
        rhs = `${name}.${value.field}`;
      } else {
        rhs = kind === 'In' ? `(:...${sym})` : `:${sym}`;
        param[sym] = value;
      }

      let clause: string;
      switch (kind) {
        case 'Eq': {
          clause = `${name}.${field} = ${rhs}`;
          break;
        }
        case 'Neq': {
          clause = `${name}.${field} <> ${rhs}`;
          break;
        }
        case 'In': {
          clause = `${name}.${field} IN ${rhs}`;
          break;
        }
        default:
          throw new Error(`Unknown constraint kind: ${kind}`);
      }

      return query.andWhere(clause, param);
    };

    return (constraints: Filter[]) =>
      constraints.reduce(constrain, repo.createQueryBuilder(name));
  };

  type Resource =
    | User
    | Repo
    | Org
    | Issue
    | RepoRole
    | OrgRole
    | Bar
    | Foo
    | Num;

  const execQuery = (q: SelectQueryBuilder<Resource>) => q.getMany();
  const combineQuery = <T extends SelectQueryBuilder<Resource>>(a: T, b: T) => {
    // this is kind of bad but typeorm doesn't give you a lot of tools
    // for working with queries :(
    const whereClause = (sql: string) => /WHERE (.*)$/.exec(sql)?.[1];
    const bClause = whereClause(b.getQuery());
    if (bClause) a = a.orWhere(bClause, b.getParameters());
    const aClause = whereClause(a.getQuery());
    if (aClause) a = a.where(`(${aClause})`, a.getParameters());
    return a;
  };

  // set global exec/combine query functions
  oso.setDataFilteringQueryDefaults({ execQuery, combineQuery });

  oso.registerClass(User, {
    buildQuery: fromRepo(users, 'user'),
    fields: {
      id: Number,
      email: String,
      repoRoles: new Relation('many', 'RepoRole', 'id', 'userId'),
      orgRoles: new Relation('many', 'OrgRole', 'id', 'userId'),
    },
  });

  oso.registerClass(Repo, {
    buildQuery: fromRepo(repos, 'repo'),
    fields: {
      id: Number,
      name: String,
      orgId: Number,
      org: new Relation('one', 'Org', 'orgId', 'id'),
      roles: new Relation('many', 'RepoRole', 'id', 'repoId'),
      issues: new Relation('many', 'Issue', 'id', 'repoId'),
    },
  });

  oso.registerClass(Org, {
    buildQuery: fromRepo(orgs, 'org'),
    fields: {
      id: Number,
      name: String,
      billing_address: String,
      base_repo_role: String,
      repos: new Relation('many', 'Repo', 'id', 'orgId'),
      roles: new Relation('many', 'OrgRole', 'id', 'orgId'),
    },
  });

  oso.registerClass(Issue, {
    buildQuery: fromRepo(issues, 'issue'),
    fields: {
      id: Number,
      title: String,
      repoId: Number,
      repo: new Relation('one', 'Repo', 'repoId', 'id'),
    },
  });

  oso.registerClass(RepoRole, {
    buildQuery: fromRepo(repoRoles, 'repo_role'),
    fields: {
      id: Number,
      role: String,
      repoId: Number,
      userId: Number,
      user: new Relation('one', 'User', 'userId', 'id'),
      repo: new Relation('one', 'Repo', 'repoId', 'id'),
    },
  });

  oso.registerClass(OrgRole, {
    buildQuery: fromRepo(orgRoles, 'org_role'),
    fields: {
      id: Number,
      role: String,
      orgId: Number,
      userId: Number,
      user: new Relation('one', 'User', 'userId', 'id'),
      org: new Relation('one', 'Org', 'orgId', 'id'),
    },
  });

  oso.registerClass(Bar, {
    buildQuery: fromRepo(bars, 'bar'),
    fields: {
      id: String,
      isCool: Boolean,
      isStillCool: Boolean,
      foos: new Relation('many', 'Foo', 'id', 'barId'),
    },
  });

  oso.registerClass(Foo, {
    buildQuery: fromRepo(foos, 'foo'),
    fields: {
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
    fields: {
      id: String,
      fooId: String,
      data: String,
      foo: new Relation('one', 'Foo', 'fooId', 'id'),
    },
  });

  oso.registerClass(Num, {
    buildQuery: fromRepo(nums, 'num'),
    fields: {
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

  async function make<T>(r: Repository<T>, x: DeepPartial<T>): Promise<T> {
    return await r.findOneOrFail(await r.save(x));
  }
  const pol = await make(repos, { name: 'pol', org: osohq }),
    ios = await make(repos, { name: 'ios', org: apple }),
    app = await make(repos, { name: 'app', org: tiktok }),
    bug = await make(issues, { title: 'bug', repo: pol }),
    lag = await make(issues, { title: 'lag', repo: ios }),
    steve = await make(users, { email: 'steve@osohq.com' }),
    leina = await make(users, { email: 'leina@osohq.com' }),
    gabe = await make(users, { email: 'gabe@osohq.com' }),
    gwen = await make(users, { email: 'gwen@osohq.com' });

  await orgRoles.save({ name: 'owner', org: osohq, user: leina });
  await orgRoles.save({ name: 'member', org: tiktok, user: gabe });

  await repoRoles.save({ name: 'writer', repo: ios, user: steve });
  await repoRoles.save({ name: 'reader', repo: app, user: gwen });

  const checkAuthz = async (
    actor: unknown,
    action: string,
    resource: Class,
    expected: unknown[]
  ) => {
    for (const x of expected)
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
  test('specializers', async () => {
    const { oso, checkAuthz, aFoo, aLog } = await fixtures();
    await oso.loadStr(`
      allow(foo: Foo,             "NoneNone", log) if foo = log.foo;
      allow(foo,                  "NoneCls",  log: Log) if foo = log.foo;
      allow(foo,                  "NoneDict", _: {foo:foo});
      allow(foo,                  "NonePtn",  _: Log{foo: foo});
      allow(foo: Foo,             "ClsNone",  log) if log in foo.logs;
      allow(foo: Foo,             "ClsCls",   log: Log) if foo = log.foo;
      allow(foo: Foo,             "ClsDict",  _: {foo: foo});
      allow(foo: Foo,             "ClsPtn",   _: Log{foo: foo});
      allow(_: {logs: logs},      "DictNone", log) if log in logs;
      allow(_: {logs: logs},      "DictCls",  log: Log) if log in logs;
      allow(foo: {logs: logs},    "DictDict", log: {foo: foo}) if log in logs;
      allow(foo: {logs: logs},    "DictPtn",  log: Log{foo: foo}) if log in logs;
      allow(_: Foo{logs: logs},   "PtnNone",  log) if log in logs;
      allow(_: Foo{logs: logs},   "PtnCls",   log: Log) if log in logs;
      allow(foo: Foo{logs: logs}, "PtnDict",  log: {foo: foo}) if log in logs;
      allow(foo: Foo{logs: logs}, "PtnPtn",   log: Log{foo: foo}) if log in logs;
    `);

    const parts = ['None', 'Cls', 'Dict', 'Ptn'];
    for (const p1 of parts)
      for (const p2 of parts) await checkAuthz(aFoo, p1 + p2, Log, [aLog]);
  });
  test('relations and operators', async () => {
    const { oso, checkAuthz, aFoo, anotherFoo, thirdFoo } = await fixtures();

    await oso.loadStr(`
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
    await oso.loadStr('allow("gwen", "put", _: Foo);');
    expect(await oso.authorizedResources('gwen', 'delete', Foo)).toEqual([]);
  });

  test('not equals', async () => {
    const { oso, checkAuthz, byeBar } = await fixtures();
    await oso.loadStr(`
      allow("gwen", "get", bar: Bar) if
        bar.isCool != bar.isStillCool;`);
    await checkAuthz('gwen', 'get', Bar, [byeBar]);
  });

  test('returning, modifying and executing a query', async () => {
    const { oso, aFoo, anotherFoo } = await fixtures();
    await oso.loadStr(`
      allow("gwen", "put", foo: Foo) if
        rec in foo.numbers and
        rec.number in [1, 2];`);

    const query = await oso.authorizedQuery('gwen', 'put', Foo);

    if (!(query instanceof SelectQueryBuilder)) throw new Error();

    let result = await query.getMany();
    expect(result).toHaveLength(2);
    expect(result).toEqual(expect.arrayContaining([aFoo, anotherFoo]));

    result = await query.andWhere("id = 'one'").getMany();
    expect(result).toHaveLength(1);
    expect(result).toEqual(expect.arrayContaining([aFoo]));
  });

  test('a roles policy', async () => {
    const { oso, checkAuthz, aFoo, anotherFoo, helloBar } = await fixtures();
    await oso.loadStr(`
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

  test('a gitclub-like policy', async () => {
    const { oso, checkAuthz, gwen, lag, steve, gabe, leina, pol, app, ios } =
      await fixtures();
    await oso.loadStr(`
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

resource Issue {
  permissions = ["read"];
  relations = { parent: Repo };

  "read" if "reader" on "parent";
}

allow(actor, action, resource) if
  has_permission(actor, action, resource);

# Users can see each other.
has_permission(_: User, "read", _: User);

# A User can read their own profile.
has_permission(user: User, "read_profile", user: User);

# Any logged-in user can create a new org.
has_permission(_: User, "create", _: Org);

has_role(user: User, name: String, org: Org) if
    role in user.orgRoles and
    role matches { name: name, org: org };

has_role(user: User, name: String, repo: Repo) if
    role in user.repoRoles and
    role matches { name: name, repo: repo };

has_relation(org: Org, "parent", _: Repo{org: org});
has_relation(repo: Repo, "parent", _: Issue{repo: repo});

    `);

    await checkAuthz(steve, 'create_issues', Repo, [ios]);
    await checkAuthz(steve, 'read', Issue, [lag]);
    await checkAuthz(gwen, 'read', Repo, [app]);
    await checkAuthz(gwen, 'read', Issue, []);
    await checkAuthz(gwen, 'create_issues', Repo, []);
    await checkAuthz(leina, 'create_issues', Repo, [pol]);
    await checkAuthz(gabe, 'create_issues', Repo, []);
  });
});
