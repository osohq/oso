import type { Host } from './Host';
import type { PolarComparisonOperator, HostTypes, obj } from './types';
declare type RelationKind = 'one' | 'many';
/** Represents relationships between two resources, eg. one-one or one-many. */
export declare class Relation {
    kind: RelationKind;
    otherType: string;
    myField: string;
    otherField: string;
    constructor(kind: RelationKind, otherType: string, myField: string, otherField: string);
    serialize(): obj;
}
export interface Filter {
    model: string;
    relations: FilterRelation[];
    conditions: FilterCondition[][];
    types: HostTypes;
}
interface FilterRelation {
    fromTypeName: string;
    fromFieldName: string;
    toTypeName: string;
}
export interface FilterCondition {
    lhs: Datum;
    cmp: PolarComparisonOperator;
    rhs: Datum;
}
export declare type Datum = Projection | Immediate;
export interface Immediate {
    value: unknown;
}
export interface Projection {
    typeName: string;
    fieldName?: string;
}
export declare function isProjection(x: unknown): x is Projection;
export interface Adapter<Query, Resource> {
    buildQuery: (f: Filter) => Query;
    executeQuery: (q: Query) => Promise<Resource[]>;
}
export interface FilterJson {
    conditions: unknown[][][];
    relations: string[][];
    root: string;
}
export declare function parseFilter<Query, Resource>(filter_json: FilterJson, host: Host<Query, Resource>): Promise<Filter>;
export {};
