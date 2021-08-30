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

describe('Data filtering', () => {
  test('using typeorm with sqlite', async () => {
    const connection = await createConnection({
      type: 'sqlite',
      database: `:memory:`,
      entities: [Foo, Bar, Num],
      synchronize: true,
      logging: false,
    });

    let bars = connection.getRepository(Bar);
    let foos = connection.getRepository(Foo);
    let nums = connection.getRepository(Num);

    async function mkBar(id: string, cool: boolean, stillCool: boolean) {
      let bar = new Bar();
      bar.id = id;
      bar.isCool = cool;
      bar.isStillCool = stillCool;
      await bars.save(bar);
      return bar;
    }

    async function mkFoo(id: string, barId: string, fooey: boolean) {
      let foo = new Foo();
      foo.id = id;
      foo.barId = barId;
      foo.isFooey = fooey;
      await foos.save(foo);
      return foo;
    }

    async function mkNum(number: number, fooId: string) {
      let num = new Num();
      num.fooId = fooId;
      num.number = number;
      await nums.save(num);
      return num;
    }

    let helloBar = await mkBar('hello', true, true);
    let byeBar = await mkBar('goodbye', true, false);

    let aFoo = await mkFoo('one', 'hello', false);
    let anotherFoo = await mkFoo('another', 'hello', true);
    let thirdFoo = await mkFoo('next', 'goodbye', true);

    await mkNum(0, 'one');
    await mkNum(1, 'one');
    await mkNum(2, 'one');

    await mkNum(0, 'another');
    await mkNum(1, 'another');

    await mkNum(0, 'next');

    const oso = new Oso();

    let i = 0;
    const gensym = (tag: any) => `_${tag}_${i++}`;

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

    // this is kind of bad but typeorm doesn't let you do very much with queries ...
    const whereClause = (sql: string) => /WHERE (.*)$/.exec(sql)![1];

    const execQuery = (q: any) => q.getMany();
    const combineQuery = (a: any, b: any) => {
      a = a.orWhere(whereClause(b.getQuery()), b.getParameters());
      return a.where(`(${whereClause(a.getQuery())})`, a.getParameters());
    };

    const barType = new Map();
    barType.set('id', String);
    barType.set('isCool', Boolean);
    barType.set('isStillCool', Boolean);
    barType.set('foos', new Relationship('children', 'Foo', 'id', 'barId'));
    oso.registerClass(Bar, {
      name: 'Bar',
      types: barType,
      buildQuery: fromRepo(bars, 'bar'),
      execQuery: execQuery,
      combineQuery: combineQuery,
    });

    const fooType = new Map();
    fooType.set('id', String);
    fooType.set('barId', String);
    fooType.set('isFooey', Boolean);
    fooType.set('bar', new Relationship('parent', 'Bar', 'barId', 'id'));
    fooType.set('numbers', new Relationship('children', 'Num', 'id', 'fooId'));
    oso.registerClass(Foo, {
      name: 'Foo',
      types: fooType,
      buildQuery: fromRepo(foos, 'foo'),
      execQuery: execQuery,
      combineQuery: combineQuery,
    });

    const numType = new Map();
    numType.set('number', Number);
    numType.set('fooId', String);
    numType.set('foo', new Relationship('parent', 'Foo', 'fooId', 'id'));
    oso.registerClass(Num, {
      name: 'Num',
      types: numType,
      buildQuery: fromRepo(nums, 'num'),
      execQuery: execQuery,
      combineQuery: combineQuery,
    });

    const expectSameResults = (a: any[], b: any[]) => {
      expect(a).toEqual(expect.arrayContaining(b));
      expect(b).toEqual(expect.arrayContaining(a));
    };

    const checkAuthz = async (
      actor: any,
      action: string,
      resource: any,
      expected: any[]
    ) => {
      for (let x in expected)
        expect(await oso.isAllowed(actor, action, expected[x])).toBe(true);
      expectSameResults(
        await oso.authorizedResources(actor, action, resource),
        expected
      );
    };

    oso.loadStr(`
      allow("steve", "get", resource: Foo) if
          resource.bar = bar and
          bar.isCool = true and
          resource.isFooey = true;
      allow("steve", "patch", foo: Foo) if
        foo in foo.bar.foos;`);
    await checkAuthz('steve', 'get', Foo, [anotherFoo, thirdFoo]);
    await checkAuthz('steve', 'patch', Foo, [aFoo, anotherFoo, thirdFoo]);

    oso.loadStr(`
      allow(num: Integer, "count", foo: Foo) if
        rec in foo.numbers and
        rec.number = num;
      allow("gwen", "put", foo: Foo) if
        rec in foo.numbers and
        rec.number in [1, 2];
      allow("gwen", "get", bar: Bar) if
        bar.isCool != bar.isStillCool;`);
    await checkAuthz(0, 'count', Foo, [aFoo, anotherFoo, thirdFoo]);
    await checkAuthz(1, 'count', Foo, [aFoo, anotherFoo]);
    await checkAuthz(2, 'count', Foo, [aFoo]);

    await checkAuthz('gwen', 'put', Foo, [aFoo, anotherFoo]);
    await checkAuthz('gwen', 'get', Bar, [byeBar]);
    await checkAuthz('gwen', 'delete', Foo, []);

    let putQuery = await oso.authorizedQuery('gwen', 'put', Foo);
    expect(await putQuery.andWhere("id = 'one'").getMany()).toEqual([aFoo]);

    oso.clearRules();
    oso.loadStr(`
          resource(_type: Bar, "bar", actions, roles) if
              actions = ["get"] and
              roles = {
                  owner: {
                      permissions: ["get"],
                      implies: ["foo:reader"]
                  }
              };

          resource(_type: Foo, "foo", actions, roles) if
              actions = ["read"] and
              roles = {
                  reader: {
                      permissions: ["read"]
                  }
              };

          parent_child(parent_bar: Bar, foo: Foo) if
              foo.bar = parent_bar;

          actor_has_role_for_resource("steve", "owner", bar: Bar) if bar.id = "hello";

          allow(actor, action, resource) if role_allows(actor, action, resource);
      `);
    oso.enableRoles();
    await checkAuthz('steve', 'get', Bar, [helloBar]);
    await checkAuthz('steve', 'read', Foo, [aFoo, anotherFoo]);
  });
});
