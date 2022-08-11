"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.typeOrmAdapter = void 0;
const filter_1 = require("./filter");
// helpers for writing SQL
const ops = { Eq: '=', Geq: '>=', Gt: '>', Leq: '<=', Lt: '<', Neq: '!=' };
const orClauses = (clauses) => clauses.length === 0 ? '1=0' : `(${clauses.join(' OR ')})`;
const andClauses = (clauses) => clauses.length === 0 ? '1=1' : `(${clauses.join(' AND ')})`;
// Expand conditions like "user = #<user id=12>" to "user.id = 12"
// Only the ORM knows how to do this, so we need to do it here.
const expandObjectComparison = (c) => {
    for (const { a, b } of [
        { a: 'lhs', b: 'rhs' },
        { a: 'rhs', b: 'lhs' },
    ]) {
        const q = c[a];
        if (filter_1.isProjection(q) && q.fieldName === undefined)
            return {
                [a]: { typeName: q.typeName, fieldName: 'id' },
                cmp: c.cmp,
                [b]: { value: c[b].value.id },
            };
    }
    return c;
};
function typeOrmAdapter(connection) {
    return {
        executeQuery: (query) => query.getMany(),
        buildQuery: ({ model, conditions, relations, types, }) => {
            // make a query builder
            const queryBuilder = connection
                .getRepository(model)
                .createQueryBuilder(model);
            // join all the tables in the relation
            const relation = relations.reduce((query, { fromTypeName, fromFieldName, toTypeName }) => {
                // extract the fields we're joining on
                const { myField, otherField } = types.get(fromTypeName).fields.get(fromFieldName);
                // write the join condition
                const join = `${fromTypeName}.${myField} = ${toTypeName}.${otherField}`;
                return query.innerJoin(toTypeName, toTypeName, join);
            }, queryBuilder);
            // now write the where clause
            //
            // condition to sql
            const sqlCondition = (c) => {
                c = expandObjectComparison(c);
                return `${sqlData(c.lhs)} ${ops[c.cmp]} ${sqlData(c.rhs)}`;
            };
            // for storing interpolated values
            const values = {};
            // data to sql. calling this on filter data populates the values object
            const sqlData = (d) => {
                if (filter_1.isProjection(d))
                    return `${d.typeName}.${d.fieldName}`;
                const key = Object.keys(values).length;
                values[key] = d.value;
                return `:${key}`;
            };
            const whereClause = orClauses(conditions.map(ands => andClauses(ands.map(sqlCondition))));
            return relation.where(whereClause, values);
        },
    };
}
exports.typeOrmAdapter = typeOrmAdapter;
//# sourceMappingURL=typeOrmAdapter.js.map