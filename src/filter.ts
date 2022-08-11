import type { Host } from './Host';
import type {
  PolarComparisonOperator,
  PolarValue,
  HostTypes,
  obj,
} from './types';
import { OsoError } from './errors';

type RelationKind = 'one' | 'many';

/** Represents relationships between two resources, eg. one-one or one-many. */
export class Relation {
  kind: RelationKind;
  otherType: string;
  myField: string;
  otherField: string;

  constructor(
    kind: RelationKind,
    otherType: string,
    myField: string,
    otherField: string
  ) {
    this.kind = kind;
    this.otherType = otherType;
    this.myField = myField;
    this.otherField = otherField;
  }

  serialize(): obj {
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

// Represents an abstract query over a data source
export interface Filter {
  model: string; // type of query / source of data
  relations: FilterRelation[]; // named relations to other data sources
  conditions: FilterCondition[][]; // query conditions: an OR of ANDs
  types: HostTypes; // type information for use by the adapter (see below)
}

// Represents a named relation between two data sources, eg. an organization to its members
interface FilterRelation {
  fromTypeName: string;
  fromFieldName: string;
  toTypeName: string;
}

// Represents a boolean condition over a set of data sources.
export interface FilterCondition {
  lhs: Datum;
  cmp: PolarComparisonOperator;
  rhs: Datum;
}

// Data in conditions can be immediate values (strings, numbers, etc) or "projections"
export type Datum = Projection | Immediate;

export interface Immediate {
  value: unknown;
}

// A projection is a type and an optional field, like "User.name", "Post.user_id", or "Tag".
// If the field name is absent, the adapter should substitute the primary key (eg. "Tag"
// becomes "Tag.id")
export interface Projection {
  typeName: string;
  fieldName?: string;
}

export function isProjection(x: unknown): x is Projection {
  return (x as Projection).typeName !== undefined;
}

// An Adapter can send a Filter to a query, and a query to a list of resources.
export interface Adapter<Query, Resource> {
  buildQuery: (f: Filter) => Query;
  executeQuery: (q: Query) => Promise<Resource[]>;
}

export interface FilterJson {
  conditions: unknown[][][];
  relations: string[][];
  root: string;
}

export async function parseFilter<Query, Resource>(
  filter_json: FilterJson,
  host: Host<Query, Resource>
): Promise<Filter> {
  const filter = {
    model: filter_json.root,
    relations: [] as FilterRelation[],
    conditions: [] as FilterCondition[][],
    types: host.types,
  };

  for (const [fromTypeName, fromFieldName, toTypeName] of filter_json.relations)
    filter.relations.push({ fromTypeName, fromFieldName, toTypeName });

  async function parseDatum(
    d: obj,
    host: Host<Query, Resource>
  ): Promise<Datum> {
    const k = Object.getOwnPropertyNames(d)[0];
    switch (k) {
      case 'Field': {
        const [typeName, fieldName] = d[k] as string[];
        return { typeName, fieldName };
      }
      case 'Immediate':
        return { value: await host.toJs({ value: d[k] as PolarValue }) };
      default: {
        throw new OsoError('Invalid filter json.');
      }
    }
  }

  for (const cs of filter_json.conditions) {
    const and_group: FilterCondition[] = [];
    for (const [l, op, r] of cs) {
      const condition = {
        lhs: await parseDatum(l as obj, host),
        cmp: op as PolarComparisonOperator,
        rhs: await parseDatum(r as obj, host),
      };
      and_group.push(condition);
    }
    filter.conditions.push(and_group);
  }

  return filter;
}
