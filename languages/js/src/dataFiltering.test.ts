import { Oso } from './Oso';
import { Relationship } from './dataFiltering';
import "reflect-metadata"
import { Entity, PrimaryColumn, Column, createConnection } from "typeorm";

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
        type: "sqlite",
        database: `:memory:`,
        entities: [Foo, Bar],
        synchronize: true,
        logging: false
    });
    let bars = connection.getRepository(Bar);

    let helloBar = new Bar();
    helloBar.id = "hello";
    helloBar.isCool = true;
    helloBar.isStillCool = true;

    await bars.save(helloBar)

    let foos = connection.getRepository(Foo);

    let anotherFoo = new Foo();
    anotherFoo.id = "Another";
    anotherFoo.barId = "hello";
    anotherFoo.isFooey = true;

    await foos.save(anotherFoo);

    const oso = new Oso();

    function fromRepo(repo: any, name: string, constraints: any) {
        let query = repo.createQueryBuilder(name);
        let addedWhere = false;
        for (let i in constraints) {
            let c = constraints[i];
            let clause;
            switch (c.kind) {
                case "Eq": {
                    clause = `${name}.${c.field} = :${c.field}`
                } break;
                case "In": {
                    clause = `${name}.${c.field} IN (:...${c.field})`
                } break;
            }
            let param: any = {}
            param[c.field] = c.value
            if (!addedWhere) {
                query.where(clause, param)
                addedWhere = true;
            } else {
                query.andWhere(clause, param)
            }
        }
        return query.getMany()
    }

    function getBars(constraints: any) {
        return fromRepo(bars, "bar", constraints)
    }

    function getFoos(constraints: any) {
        return fromRepo(foos, "foo", constraints)
    }

    const barType = new Map();
    barType.set("id", String)
    barType.set("isCool", Boolean)
    barType.set("isStillCool", Boolean)
    oso.registerClass(Bar, "Bar", barType, getBars);

    const fooType = new Map();
    fooType.set("id", String)
    fooType.set("barId", String)
    fooType.set("isFooey", Boolean)
    fooType.set("bar", new Relationship("parent", "Bar", "barId", "id"))
    oso.registerClass(Foo, "Foo", fooType, getFoos);

    oso.loadStr(`
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.isCool = true and
            resource.isFooey = true;
    `)
    expect(await oso.isAllowed("steve", "get", anotherFoo)).toBe(true);
    expect(await oso.getAllowedResources("steve", "get", Foo)).toEqual([anotherFoo]);
});