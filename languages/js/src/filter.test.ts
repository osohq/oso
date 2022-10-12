import { Oso } from './Oso';
import { Relation } from './filter';
import 'reflect-metadata';
import {
  OneToMany,
  ManyToOne,
  PrimaryGeneratedColumn,
  Entity,
  PrimaryColumn,
  Column,
  createConnection,
  Connection,
  Repository,
  SelectQueryBuilder,
  DeepPartial,
} from 'typeorm';
import { typeOrmAdapter } from './typeOrmAdapter';
import { Class } from './types';

class TestOso<R, A> extends Oso<
  A,
  string | number,
  R,
  unknown,
  unknown,
  SelectQueryBuilder<R>
> {
  async checkAuthz(
    actor: A,
    action: string | number,
    resource: Class<R>,
    expected: R[]
  ) {
    for (const x of expected)
      expect(await this.isAllowed(actor, action, x)).toBe(true);
    const actual = await this.authorizedResources(actor, action, resource);

    expect(actual).toHaveLength(expected.length);
    expect(actual).toEqual(expect.arrayContaining(expected));
  }
}

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
  @Column({ nullable: true })
  subtitle!: string;
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
  const connection: Connection = await createConnection({
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

  async function mkBar(id: string, isCool: boolean, isStillCool: boolean) {
    return await bars.findOneOrFail(
      await bars.save({
        id,
        isCool,
        isStillCool,
      })
    );
  }

  async function mkFoo(id: string, barId: string, isFooey: boolean) {
    return await foos.findOneOrFail(
      await foos.save({
        id,
        barId,
        isFooey,
      })
    );
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
  const hersheyBar = await mkBar('hershey', false, false);

  const somethingFoo = await mkFoo('something', 'hello', false);
  const anotherFoo = await mkFoo('another', 'hello', true);
  const thirdFoo = await mkFoo('third', 'hello', true);
  const fourthFoo = await mkFoo('fourth', 'goodbye', true);

  const aLog = await logs.findOneOrFail(
    await logs.save({ id: 'a', fooId: 'fourth', data: 'goodbye' })
  );
  const anotherLog = await logs.findOneOrFail(
    await logs.save({ id: 'b', fooId: 'another', data: 'world' })
  );
  const thirdLog = await logs.findOneOrFail(
    await logs.save({ id: 'c', fooId: 'third', data: 'steve' })
  );

  const allNums: Num[] = [];

  for (const i of [0, 1, 2]) allNums.push(await mkNum(i, 'something'));
  for (const i of [0, 1]) allNums.push(await mkNum(i, 'another'));
  for (const i of [0]) allNums.push(await mkNum(i, 'third'));

  type Resource =
    | User
    | Repo
    | Org
    | Issue
    | RepoRole
    | OrgRole
    | Bar
    | Foo
    | Num
    | Log;

  const oso = new TestOso<Resource, unknown>();

  oso.setDataFilteringAdapter(typeOrmAdapter(connection));

  oso.registerClass(User, {
    fields: {
      id: Number,
      email: String,
      repoRoles: new Relation('many', 'RepoRole', 'id', 'userId'),
      orgRoles: new Relation('many', 'OrgRole', 'id', 'userId'),
    },
  });

  oso.registerClass(Repo, {
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
    fields: {
      id: Number,
      title: String,
      subtitle: String,
      repoId: Number,
      repo: new Relation('one', 'Repo', 'repoId', 'id'),
    },
  });

  oso.registerClass(RepoRole, {
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
    fields: {
      id: String,
      isCool: Boolean,
      isStillCool: Boolean,
      foos: new Relation('many', 'Foo', 'id', 'barId'),
    },
  });

  oso.registerClass(Foo, {
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
    fields: {
      id: String,
      fooId: String,
      data: String,
      foo: new Relation('one', 'Foo', 'fooId', 'id'),
    },
  });

  oso.registerClass(Num, {
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
  );
  const osohq = await orgs.findOneOrFail(
    await orgs.save({
      name: 'osohq',
      billing_address: 'new york, NY',
      base_repo_role: 'reader',
    })
  );
  const tiktok = await orgs.findOneOrFail(
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
    lag = await make(issues, { title: 'lag', subtitle: 'fix', repo: ios }),
    steve = await make(users, { email: 'steve@osohq.com' }),
    leina = await make(users, { email: 'leina@osohq.com' }),
    gabe = await make(users, { email: 'gabe@osohq.com' }),
    gwen = await make(users, { email: 'gwen@osohq.com' });

  await orgRoles.save({ name: 'owner', org: osohq, user: leina });
  await orgRoles.save({ name: 'member', org: tiktok, user: gabe });

  await repoRoles.save({ name: 'writer', repo: ios, user: steve });
  await repoRoles.save({ name: 'reader', repo: app, user: gwen });

  const allFoos = [somethingFoo, anotherFoo, thirdFoo, fourthFoo];
  const allBars = [helloBar, byeBar, hersheyBar];
  const allLogs = [aLog, anotherLog, thirdLog];
  return {
    oso,
    somethingFoo,
    anotherFoo,
    thirdFoo,
    fourthFoo,
    foos: allFoos,
    nums: allNums,
    aLog,
    anotherLog,
    thirdLog,
    logs: allLogs,
    helloBar,
    byeBar,
    hersheyBar,
    bars: allBars,
    fooBar: (foo: Foo) => allBars.filter((bar: Bar) => bar.id === foo.barId)[0],
    barFoos: (bar: Bar) => allFoos.filter((foo: Foo) => foo.barId === bar.id),
    logFoo: (log: Log) => allFoos.filter((foo: Foo) => foo.id === log.fooId)[0],
    fooLogs: (foo: Foo) => allLogs.filter((log: Log) => log.fooId === foo.id),
    fooNums: (foo: Foo) => allNums.filter(num => num.fooId === foo.id),
    numFoo: (num: Num) => allFoos.filter(foo => foo.id === num.fooId)[0],
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

describe('Data filtering parity tests', () => {
  test('test_model', async () => {
    const { oso, somethingFoo, anotherFoo } = await fixtures();
    await oso.loadStr('allow(_, _, _: Foo{id: "something"});');
    await oso.checkAuthz('gwen', 'get', Foo, [somethingFoo]);
    oso.clearRules();
    await oso.loadStr(`
      allow(_, _, _: Foo{id: "something"});
      allow(_, _, _: Foo{id: "another"});
    `);
    await oso.checkAuthz('gwen', 'get', Foo, [somethingFoo, anotherFoo]);
  });

  test('test_authorize_scalar_attribute_eq', async () => {
    const { oso, bars, foos } = await fixtures();
    await oso.loadStr(`
      allow(_: Bar, "read", _: Foo{isFooey: true});
      allow(bar: Bar, "read", _: Foo{bar: bar});
    `);
    for (const bar of bars) {
      const expected = foos.filter(foo => foo.isFooey || foo.barId === bar.id);
      await oso.checkAuthz(bar, 'read', Foo, expected);
    }
  });

  test('test_authorize_scalar_attribute_condition', async () => {
    const { oso, bars, foos, fooBar } = await fixtures();
    await oso.loadStr(`
      allow(bar: Bar{isCool: true}, "read", _: Foo{bar: bar});
      allow(_: Bar, "read", _: Foo{bar: b, isFooey: true}) if b.isCool;
      allow(_: Bar{isStillCool: true}, "read", foo: Foo) if
        foo.bar.isCool = false;
    `);
    for (const bar of bars) {
      const expected = foos.filter(foo => {
        // typeORM fails to perform the basic functions of an ORM :|
        const myBar = fooBar(foo);
        return (
          (myBar.isCool && myBar.id === bar.id) ||
          (myBar.isCool && foo.isFooey) ||
          (!myBar.isCool && bar.isStillCool)
        );
      });
      await oso.checkAuthz(bar, 'read', Foo, expected);
    }
  });

  //  test('test_in_multiple_attribute_relationship', async () => {
  //  });

  test('test_nested_relationship_many_single', async () => {
    const { oso, logs, fooBar, logFoo } = await fixtures();
    await oso.loadStr(`
      allow(log: Log, "read", bar: Bar) if log.foo in bar.foos;
    `);
    for (const log of logs)
      await oso.checkAuthz(log, 'read', Bar, [fooBar(logFoo(log))]);
  });

  test('test_nested_relationships_many_many', async () => {
    const { oso, logs, fooBar, logFoo } = await fixtures();
    await oso.loadStr(`
      allow(log: Log, "read", bar: Bar) if
        foo in bar.foos and log in foo.logs;
    `);
    for (const log of logs)
      await oso.checkAuthz(log, 'read', Bar, [fooBar(logFoo(log))]);
  });

  test('test_nested_relationship_many_many_constrained', async () => {
    const { oso, logs, bars, barFoos } = await fixtures();
    await oso.loadStr(`
      allow(log: Log{data: "steve"}, "read", bar: Bar) if
        foo in bar.foos and log in foo.logs;
    `);
    for (const log of logs) {
      const expected = bars.filter(bar => {
        if (log.data !== 'steve') return false;
        for (const foo of barFoos(bar)) if (foo.id === log.fooId) return true;
        return false;
      });
      await oso.checkAuthz(log, 'read', Bar, expected);
    }
  });

  test('test_partial_in_collection', async () => {
    const { oso, bars, barFoos } = await fixtures();
    await oso.loadStr(`
      allow(bar, "read", foo: Foo) if foo in bar.foos;
    `);
    for (const bar of bars) {
      await oso.checkAuthz(bar, 'read', Foo, barFoos(bar));
    }
  });

  test('test_partial_isa_with_path', async () => {
    const { oso, byeBar, barFoos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, foo: Foo) if check(foo.bar);
      check(bar: Bar) if bar.id = "goodbye";
      check(foo: Foo) if foo.bar.id = "hello";
    `);
    await oso.checkAuthz('gwen', 'read', Foo, barFoos(byeBar));
  });

  test('test_no_relationships', async () => {
    const { oso, foos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, foo: Foo) if foo.isFooey;
    `);
    const expected = foos.filter((foo: Foo) => foo.isFooey);
    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_neq', async () => {
    const { oso, bars, foos } = await fixtures();
    await oso.loadStr(`
      allow(_, action, foo: Foo) if foo.bar.id != action;
    `);
    for (const bar of bars) {
      const expected = foos.filter(foo => foo.barId !== bar.id);
      await oso.checkAuthz('gwen', bar.id, Foo, expected);
    }
  });

  test('test_relationship', async () => {
    const { oso, foos, fooBar } = await fixtures();
    await oso.loadStr(`
      allow(_, "get", foo: Foo) if
        foo.bar = bar and
          bar.isCool and
          foo.isFooey;
    `);

    const expected = foos.filter(
      (foo: Foo) => fooBar(foo).isCool && foo.isFooey
    );
    await oso.checkAuthz('steve', 'get', Foo, expected);
  });

  // Joins to the same table are not yet supported in new data filtering
  xtest('test_duplex_relationship', async () => {
    const { oso, foos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, foo: Foo) if foo in foo.bar.foos;
    `);
    await oso.checkAuthz('gwen', 'get', Foo, foos);
  });

  test('test_scalar_in_list', async () => {
    const { oso, foos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Foo{bar: bar}) if bar.isCool in [true, false];
    `);
    await oso.checkAuthz('gwen', 'get', Foo, foos);
  });

  test('test_var_in_vars', async () => {
    const { oso, foos, fooLogs } = await fixtures();
    await oso.loadStr(`
      allow(_, _, foo: Foo) if
        log in foo.logs and
        log.data = "goodbye";
    `);
    const expected = foos.filter((foo: Foo) => {
      for (const log of fooLogs(foo)) if (log.data === 'goodbye') return true;
      return false;
    });
    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_specializers', async () => {
    const { oso, logs, logFoo } = await fixtures();
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
    for (const a of parts)
      for (const b of parts)
        for (const log of logs)
          await oso.checkAuthz(logFoo(log), a + b, Log, [log]);
  });

  test('test_empty_constraints_in', async () => {
    const { oso, foos, fooLogs } = await fixtures();
    await oso.loadStr(`
      allow(_, "read", foo: Foo) if _ in foo.logs;
    `);
    const expected = foos.filter((foo: Foo) => fooLogs(foo).length);
    //    console.log(expected);
    await oso.checkAuthz('gwen', 'read', Foo, expected);
    // not sure why this one is failing ...
  });

  test('test_in_with_constraints_but_no_matching_object', async () => {
    const { oso } = await fixtures();
    await oso.loadStr(`
      allow(_, "read", foo: Foo) if
        log in foo.logs and
        log.data = "nope";
    `);
    await oso.checkAuthz('gwen', 'read', Foo, []);
  });

  test('test_unify_ins', async () => {
    const { oso, bars, foos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Bar{foos: foos}) if
        foo in foos and
        goo in foos and
        foo = goo;
    `);
    const expected = bars.filter((bar: Bar) => {
      for (const foo of foos) if (foo.barId === bar.id) return true;
      return false;
    });

    await oso.checkAuthz('gwen', 'read', Bar, expected);
  });

  test('test_unify_ins_field_eq', async () => {
    const { oso, bars, barFoos } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Bar{foos:foos}) if
        foo in foos and
        goo in foos and
        foo.id = goo.id;
    `);
    const expected = bars.filter(bar => barFoos(bar).length);
    await oso.checkAuthz('gwen', 'get', Bar, expected);
  });

  test('test_var_in_value', async () => {
    const { oso, aLog, anotherLog } = await fixtures();
    await oso.loadStr(`
      allow(_, _, log: Log) if log.data in ["goodbye", "world"];
    `);

    await oso.checkAuthz('gwen', 'get', Log, [aLog, anotherLog]);
  });

  test('test_field_eq', async () => {
    const { oso, bars } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Bar{isCool: cool, isStillCool: cool});
    `);
    const expected = bars.filter(bar => bar.isCool === bar.isStillCool);
    await oso.checkAuthz('gwen', 'get', Bar, expected);
  });

  test('test_field_neq', async () => {
    const { oso, bars } = await fixtures();
    await oso.loadStr(`
      allow(_, _, bar: Bar) if bar.isCool != bar.isStillCool;
    `);
    const expected = bars.filter(bar => bar.isCool !== bar.isStillCool);
    await oso.checkAuthz('gwen', 'get', Bar, expected);
  });

  test('test_const_in_coll', async () => {
    const magic = 1,
      { oso, foos, fooNums } = await fixtures();
    oso.registerConstant(magic, 'magic');
    await oso.loadStr(`
      allow(_, _, foo: Foo) if n in foo.numbers and n.number = magic;
    `);

    const expected = foos.filter(
      foo => fooNums(foo).filter(num => num.number === 1).length
    );
    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_redundant_in_on_same_field', async () => {
    const { oso, foos, fooNums } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Foo{numbers:ns}) if
        m in ns and n in ns and
        n.number = 2 and m.number = 1;
    `);

    const expected = foos.filter((foo: Foo) =>
      fooNums(foo).some((num: Num) => [1, 2].every(n => n === num.number))
    );

    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_ground_object_in_collection', async () => {
    const { oso, foos, fooNums } = await fixtures();
    await oso.loadStr(`
      allow(_, _, _: Foo{numbers:ns}) if
        n in ns and m in ns and
        n.number = 1 and m.number = 2;
    `);

    const expected = foos.filter((foo: Foo) =>
      fooNums(foo).some((num: Num) => [1, 2].every(n => n === num.number))
    );

    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_param_field', async () => {
    const { oso, logs } = await fixtures();
    await oso.loadStr(`
      allow(data, id, _: Log{data: data, id: id});
    `);
    for (const log of logs) await oso.checkAuthz(log.data, log.id, Log, [log]);
  });

  test('test_field_cmp_rel_field', async () => {
    const { oso, foos, fooBar } = await fixtures();
    await oso.loadStr(`
      allow(_, _, foo: Foo) if foo.bar.isCool = foo.isFooey;
    `);

    const expected = foos.filter(foo => fooBar(foo).isCool === foo.isFooey);
    await oso.checkAuthz('gwen', 'get', Foo, expected);
  });

  test('test_field_cmp_rel_rel_field', async () => {
    const { oso, logs, logFoo, fooBar } = await fixtures();
    await oso.loadStr(`
      allow(_, _, log: Log) if log.data = log.foo.bar.id;
    `);
    const expected = logs.filter(log => fooBar(logFoo(log)).id === log.data);
    await oso.checkAuthz('gwen', 'get', Log, expected);
  });

  // FIXME failing tests ????

  test('test_parent_child_cases', async () => {
    const { oso, logs, logFoo } = await fixtures();
    await oso.loadStr(`
      allow(_: Log{foo: foo},   0, foo: Foo);
      allow(log: Log,           1, _: Foo{logs: logs}) if log in logs;
      allow(log: Log{foo: foo}, 2, foo: Foo{logs: logs}) if log in logs;
    `);
    for (const i of [0, 1, 2])
      for (const log of logs) await oso.checkAuthz(log, i, Foo, [logFoo(log)]);
  });
});

describe('Data filtering using typeorm/sqlite', () => {
  // multiple joins to the same table are not yet supported in the new data filtering
  xtest('relations and operators', async () => {
    const { oso, somethingFoo, anotherFoo, thirdFoo, fourthFoo } =
      await fixtures();

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

    await oso.checkAuthz('steve', 'get', Foo, [
      anotherFoo,
      thirdFoo,
      fourthFoo,
    ]);
    await oso.checkAuthz('steve', 'patch', Foo, [
      somethingFoo,
      anotherFoo,
      thirdFoo,
      fourthFoo,
    ]);

    await oso.checkAuthz(0, 'count', Foo, [somethingFoo, anotherFoo, thirdFoo]);
    await oso.checkAuthz(1, 'count', Foo, [somethingFoo, anotherFoo]);
    await oso.checkAuthz(2, 'count', Foo, [somethingFoo]);
  });

  test('an empty result', async () => {
    const { oso } = await fixtures();
    await oso.loadStr('allow("gwen", "put", _: Foo);');
    expect(await oso.authorizedResources('gwen', 'delete', Foo)).toEqual([]);
  });

  test('nil in policy', async () => {
    const { oso, bug, lag } = await fixtures();
    await oso.loadStr(`
    allow("steve", "read", issue: Issue) if
      (issue.title = "bug" and  issue.subtitle = nil) 
      or
      (issue.title = "lag" and issue.subtitle != nil)
      ;`);
    expect(await oso.authorizedResources('steve', 'read', Issue)).toEqual(
      expect.arrayContaining([bug, lag])
    );
  });

  test('not equals', async () => {
    const { oso, byeBar } = await fixtures();
    await oso.loadStr(`
      allow("gwen", "get", bar: Bar) if
        bar.isCool != bar.isStillCool;`);
    await oso.checkAuthz('gwen', 'get', Bar, [byeBar]);
  });

  test('returning, modifying and executing a query', async () => {
    const { oso, somethingFoo, anotherFoo } = await fixtures();
    await oso.loadStr(`
      allow("gwen", "put", foo: Foo) if
        rec in foo.numbers and
        rec.number in [1, 2];`);

    const query = await oso.authorizedQuery('gwen', 'put', Foo);

    if (!(query instanceof SelectQueryBuilder)) throw new Error();

    let result = await query.getMany();
    expect(result).toHaveLength(2);
    expect(result).toEqual(expect.arrayContaining([somethingFoo, anotherFoo]));

    result = await query.andWhere("id = 'something'").getMany();
    expect(result).toHaveLength(1);
    expect(result).toEqual(expect.arrayContaining([somethingFoo]));
  });

  test('a roles policy', async () => {
    const { oso, somethingFoo, anotherFoo, thirdFoo, helloBar } =
      await fixtures();
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
    await oso.checkAuthz('steve', 'get', Bar, [helloBar]);
    await oso.checkAuthz('steve', 'read', Foo, [
      somethingFoo,
      anotherFoo,
      thirdFoo,
    ]);
  });

  test('a gitclub-like policy', async () => {
    const { oso, gwen, lag, steve, gabe, leina, pol, app, ios } =
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

    await oso.checkAuthz(steve, 'create_issues', Repo, [ios]);
    await oso.checkAuthz(steve, 'read', Issue, [lag]);
    await oso.checkAuthz(gwen, 'read', Repo, [app]);
    await oso.checkAuthz(gwen, 'read', Issue, []);
    await oso.checkAuthz(gwen, 'create_issues', Repo, []);
    await oso.checkAuthz(leina, 'create_issues', Repo, [pol]);
    await oso.checkAuthz(gabe, 'create_issues', Repo, []);
  }, 60000);
});
