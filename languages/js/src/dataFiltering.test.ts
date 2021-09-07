import { Oso } from './Oso';
import { Relation } from './dataFiltering';
import 'reflect-metadata';
import { Entity, PrimaryColumn, Column, createConnection } from 'typeorm';

@Entity()
export class Bar {
  @PrimaryColumn()
  id!: string;

  @Column()
  isCool!: boolean;

  @Column()
  isStillCool!: boolean;
}

@Entity()
export class Foo {
  @PrimaryColumn()
  id!: string;

  @Column()
  barId!: string;

  @Column()
  isFooey!: boolean;
}

@Entity()
export class Num {
  @PrimaryColumn()
  fooId!: string;
  @PrimaryColumn()
  number!: number;
}

test('data filtering', async () => {
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

  function fromRepo(repo: any, name: string, constraints: any) {
    let query = repo.createQueryBuilder(name);
    for (let i in constraints) {
      let c = constraints[i];
      let clause;
      switch (c.kind) {
        case 'Eq':
          {
            clause = `${name}.${c.field} = :${c.field}`;
          }
          break;
        case 'In':
          {
            clause = `${name}.${c.field} IN (:...${c.field})`;
          }
          break;
      }
      let param: any = {};
      param[c.field] = c.value;
      query.andWhere(clause, param);
    }
    return query.getMany();
  }

  function getBars(constraints: any) {
    return fromRepo(bars, 'bar', constraints);
  }

  function getFoos(constraints: any) {
    return fromRepo(foos, 'foo', constraints);
  }

  function getNums(constraints: any) {
    return fromRepo(nums, 'num', constraints);
  }

  const barType = new Map();
  barType.set('id', String);
  barType.set('isCool', Boolean);
  barType.set('isStillCool', Boolean);
  barType.set('foos', new Relation('many', 'Foo', 'id', 'barId'));
  oso.registerClass(Bar, 'Bar', barType, getBars);

  const fooType = new Map();
  fooType.set('id', String);
  fooType.set('barId', String);
  fooType.set('isFooey', Boolean);
  fooType.set('bar', new Relation('one', 'Bar', 'barId', 'id'));
  fooType.set('numbers', new Relation('many', 'Num', 'id', 'fooId'));
  oso.registerClass(Foo, 'Foo', fooType, getFoos);

  const numType = new Map();
  numType.set('number', Number);
  numType.set('fooId', String);
  numType.set('foo', new Relation('one', 'Foo', 'fooId', 'id'));
  oso.registerClass(Num, 'Num', numType, getNums);

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
      await oso.getAllowedResources(actor, action, resource),
      expected
    );
  };

  oso.loadStr(`
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.isCool = true and
            resource.isFooey = true;
    `);
  await checkAuthz('steve', 'get', Foo, [anotherFoo, thirdFoo]);

  oso.loadStr(`
        allow("steve", "patch", foo: Foo) if
          foo in foo.bar.foos;
    `);
  await checkAuthz('steve', 'patch', Foo, [aFoo, anotherFoo, thirdFoo]);

  oso.loadStr(`
        allow(num: Integer, "count", foo: Foo) if
          rec in foo.numbers and
          rec.number = num;
        allow("gwen", "eat", foo: Foo) if
          rec in foo.numbers and
          rec.number in [1, 2];
  `);
  await checkAuthz(0, 'count', Foo, [aFoo, anotherFoo, thirdFoo]);
  await checkAuthz(1, 'count', Foo, [aFoo, anotherFoo]);
  await checkAuthz(2, 'count', Foo, [aFoo]);
  await checkAuthz('gwen', 'eat', Foo, [aFoo, anotherFoo]);

  oso.clearRules();
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
