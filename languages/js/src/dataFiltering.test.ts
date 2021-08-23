import { Oso } from './Oso';
import { Relationship } from './dataFiltering';
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

test('data filtering', async () => {
  const connection = await createConnection({
    type: 'sqlite',
    database: `:memory:`,
    entities: [Foo, Bar],
    synchronize: true,
    logging: false,
  });

  let bars = connection.getRepository(Bar);
  let foos = connection.getRepository(Foo);

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

  let helloBar = await mkBar('hello', true, true);
  let byeBar = await mkBar('goodbye', true, false);

  let aFoo = await mkFoo('one', 'hello', false);
  let anotherFoo = await mkFoo('another', 'hello', true);
  let thirdFoo = await mkFoo('next', 'goodbye', true);

  const oso = new Oso();

  function fromRepo(repo: any, name: string, constraints: any) {
    let query = repo.createQueryBuilder(name);
    let addedWhere = false;
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
      if (!addedWhere) {
        query.where(clause, param);
        addedWhere = true;
      } else {
        query.andWhere(clause, param);
      }
    }
    return query.getMany();
  }

  function getBars(constraints: any) {
    return fromRepo(bars, 'bar', constraints);
  }

  function getFoos(constraints: any) {
    return fromRepo(foos, 'foo', constraints);
  }

  const barType = new Map();
  barType.set('id', String);
  barType.set('isCool', Boolean);
  barType.set('isStillCool', Boolean);
  oso.registerClass(Bar, 'Bar', barType, getBars);

  const fooType = new Map();
  fooType.set('id', String);
  fooType.set('barId', String);
  fooType.set('isFooey', Boolean);
  fooType.set('bar', new Relationship('parent', 'Bar', 'barId', 'id'));
  oso.registerClass(Foo, 'Foo', fooType, getFoos);

  const expectSameResults = (a: any[], b: any[]) => {
    expect(a).toEqual(expect.arrayContaining(b));
    expect(b).toEqual(expect.arrayContaining(a));
  };

  const checkAuthz = async (
    actor: string,
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
