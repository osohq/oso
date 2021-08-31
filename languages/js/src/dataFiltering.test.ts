import { Oso } from './Oso';
import { Relationship, Field } from './dataFiltering';
import 'reflect-metadata';
import { Entity, PrimaryColumn, Column, createConnection } from 'typeorm';

@Entity()
class Bar {
  @PrimaryColumn()
  id!: string;

  @Column()
  isCool!: boolean;

  @Column()
  isStillCool!: boolean;
}

@Entity()
class Foo {
  @PrimaryColumn()
  id!: string;

  @Column()
  barId!: string;

  @Column()
  isFooey!: boolean;
}

@Entity()
class Num {
  @PrimaryColumn()
  fooId!: string;
  @PrimaryColumn()
  number!: number;
}

let i = 0;
const gensym = (tag?: any) => `_${tag}_${i++}`;

async function fixtures() {
  const connection = await createConnection({
    type: 'sqlite',
    database: `:memory:`,
    entities: [Foo, Bar, Num],
    synchronize: true,
    logging: false,
    name: gensym(),
  });

  const bars = connection.getRepository(Bar);
  const foos = connection.getRepository(Foo);
  const nums = connection.getRepository(Num);

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
  oso.configureDataFiltering({
    execQuery: execQuery,
    combineQuery: combineQuery,
  });

  const barType = new Map();
  barType.set('id', String);
  barType.set('isCool', Boolean);
  barType.set('isStillCool', Boolean);
  barType.set('foos', new Relationship('children', 'Foo', 'id', 'barId'));
  oso.registerClass(Bar, {
    types: barType,
    buildQuery: fromRepo(bars, 'bar'),
  });

  const fooType = new Map();
  fooType.set('id', String);
  fooType.set('barId', String);
  fooType.set('isFooey', Boolean);
  fooType.set('bar', new Relationship('parent', 'Bar', 'barId', 'id'));
  fooType.set('numbers', new Relationship('children', 'Num', 'id', 'fooId'));
  oso.registerClass(Foo, {
    types: fooType,
    buildQuery: fromRepo(foos, 'foo'),
  });

  const numType = new Map();
  numType.set('number', Number);
  numType.set('fooId', String);
  numType.set('foo', new Relationship('parent', 'Foo', 'fooId', 'id'));
  oso.registerClass(Num, {
    types: numType,
    buildQuery: fromRepo(nums, 'num'),
  });

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
    oso: oso,
    aFoo: aFoo,
    anotherFoo: anotherFoo,
    thirdFoo: thirdFoo,
    helloBar: helloBar,
    byeBar: byeBar,
    checkAuthz: checkAuthz,
  };
}

describe('Data filtering using typeorm/sqlite', () => {
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
    const { oso, checkAuthz, aFoo, anotherFoo } = await fixtures();
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

  test('a roles policy', async () => {
    const { oso, checkAuthz, aFoo, anotherFoo, helloBar } = await fixtures();
    oso.loadStr(`
      resource(_: Bar, "bar", actions, roles) if
        actions = ["get"] and
        roles = {
            owner: {
                permissions: actions,
                implies: ["foo:reader"]
            }
        };

      resource(_: Foo, "foo", actions, roles) if
        actions = ["read"] and
        roles = {
            reader: {
                permissions: actions
            }
        };

      parent_child(bar: Bar, _: Foo{bar: bar});

      actor_has_role_for_resource("steve", "owner", _: Bar{id: "hello"});

      allow(actor, action, resource) if
        role_allows(actor, action, resource);`);
    oso.enableRoles();
    await checkAuthz('steve', 'get', Bar, [helloBar]);
    await checkAuthz('steve', 'read', Foo, [aFoo, anotherFoo]);
  });
});
