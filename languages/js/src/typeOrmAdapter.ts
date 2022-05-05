import type { Connection, SelectQueryBuilder } from 'typeorm';
import { UserType, Class, obj } from './types';
import {
  Adapter,
  isProjection,
  Filter,
  Datum,
  FilterCondition,
  Immediate,
  Relation,
} from './filter';

// helpers for writing SQL
const ops = { Eq: '=', Geq: '>=', Gt: '>', Leq: '<=', Lt: '<', Neq: '!=' };
const orClauses = (clauses: string[]): string =>
  clauses.length === 0 ? '1=0' : `(${clauses.join(' OR ')})`;
const andClauses = (clauses: string[]): string =>
  clauses.length === 0 ? '1=1' : `(${clauses.join(' AND ')})`;

// Expand conditions like "user = #<user id=12>" to "user.id = 12"
// Only the ORM knows how to do this, so we need to do it here.
const expandObjectComparison = (c: FilterCondition): FilterCondition => {
  for (const { a, b } of [
    { a: 'lhs', b: 'rhs' },
    { a: 'rhs', b: 'lhs' },
  ] as { a: 'lhs' | 'rhs'; b: 'lhs' | 'rhs' }[]) {
    const q: Datum = c[a];
    if (isProjection(q) && q.fieldName === undefined)
      return {
        [a]: { typeName: q.typeName, fieldName: 'id' },
        cmp: c.cmp,
        [b]: { value: ((c[b] as Immediate).value as { id: number }).id },
      } as unknown as FilterCondition;
  }
  return c;
};

export function typeOrmAdapter<R>(
  connection: Connection
): Adapter<SelectQueryBuilder<R>, R> {
  return {
    executeQuery: (query: SelectQueryBuilder<R>) => query.getMany(),
    buildQuery: ({
      model,
      conditions,
      relations,
      types,
    }: Filter): SelectQueryBuilder<R> => {
      // make a query builder
      const queryBuilder = connection
        .getRepository(model)
        .createQueryBuilder(model);

      // join all the tables in the relation
      const relation = relations.reduce(
        (query, { fromTypeName, fromFieldName, toTypeName }) => {
          // extract the fields we're joining on
          const { myField, otherField } = (
            types.get(fromTypeName) as UserType<Class<unknown>, unknown>
          ).fields.get(fromFieldName) as Relation;
          // write the join condition
          const join = `${fromTypeName}.${myField} = ${toTypeName}.${otherField}`;
          return query.innerJoin(toTypeName, toTypeName, join);
        },
        queryBuilder as SelectQueryBuilder<R>
      );

      // now write the where clause
      //
      // condition to sql
      const sqlCondition = (c: FilterCondition): string => {
        c = expandObjectComparison(c);
        // handle null equality special case
        for (const { a, b } of [
          { a: 'lhs', b: 'rhs' },
          { a: 'rhs', b: 'lhs' },
        ] as { a: 'lhs' | 'rhs'; b: 'lhs' | 'rhs' }[]) {
          const q: Datum = c[a];
          if (!isProjection(q) && q.value === null) {
            if (c.cmp === 'Eq') {
              return `${sqlData(c[b])} is ${sqlData(q)}`;
            } else if (c.cmp === 'Neq') {
              return `${sqlData(c[b])} is not ${sqlData(q)}`;
            }
          }
        }

        return `${sqlData(c.lhs)} ${ops[c.cmp]} ${sqlData(c.rhs)}`;
      };
      // for storing interpolated values
      const values: obj = {};
      // data to sql. calling this on filter data populates the values object
      const sqlData = (d: Datum): string => {
        if (isProjection(d)) return `${d.typeName}.${d.fieldName as string}`;
        const key = Object.keys(values).length;
        values[key] = d.value;
        return `:${key}`;
      };

      const whereClause = orClauses(
        conditions.map(ands => andClauses(ands.map(sqlCondition)))
      );

      return relation.where(whereClause, values);
    },
  };
}
