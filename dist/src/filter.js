"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseFilter = exports.isProjection = exports.Relation = void 0;
const errors_1 = require("./errors");
/** Represents relationships between two resources, eg. one-one or one-many. */
class Relation {
    constructor(kind, otherType, myField, otherField) {
        this.kind = kind;
        this.otherType = otherType;
        this.myField = myField;
        this.otherField = otherField;
    }
    serialize() {
        return {
            Relation: {
                kind: this.kind,
                other_class_tag: this.otherType,
                my_field: this.myField,
                other_field: this.otherField,
            },
        };
    }
}
exports.Relation = Relation;
function isProjection(x) {
    return x.typeName !== undefined;
}
exports.isProjection = isProjection;
async function parseFilter(filter_json, host) {
    const filter = {
        model: filter_json.root,
        relations: [],
        conditions: [],
        types: host.types,
    };
    for (const [fromTypeName, fromFieldName, toTypeName] of filter_json.relations)
        filter.relations.push({ fromTypeName, fromFieldName, toTypeName });
    async function parseDatum(d, host) {
        const k = Object.getOwnPropertyNames(d)[0];
        switch (k) {
            case 'Field': {
                const [typeName, fieldName] = d[k];
                return { typeName, fieldName };
            }
            case 'Immediate':
                return { value: await host.toJs({ value: d[k] }) };
            default: {
                throw new errors_1.OsoError('Invalid filter json.');
            }
        }
    }
    for (const cs of filter_json.conditions) {
        const and_group = [];
        for (const [l, op, r] of cs) {
            const condition = {
                lhs: await parseDatum(l, host),
                cmp: op,
                rhs: await parseDatum(r, host),
            };
            and_group.push(condition);
        }
        filter.conditions.push(and_group);
    }
    return filter;
}
exports.parseFilter = parseFilter;
//# sourceMappingURL=filter.js.map