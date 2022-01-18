import type { Host } from './Host';
import type { PolarComparisonOperator, PolarValue, obj } from './types';
import { OsoError } from './errors';

export interface SerializedRelation {
  Relation: {
    kind: string;
    other_class_tag: string;
    my_field: string;
    other_field: string;
  };
}

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

  serialize(): SerializedRelation {
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

export type SerializedFields = {
  [field: string]: SerializedRelation | { Base: { class_tag: string } };
};

export interface FilterRelation {
  fromTypeName: string;
  fromFieldName: string;
  toTypeName: string;
}

export interface Projection {
  typeName: string;
  fieldName: string;
}

export interface Immediate {
  value: unknown;
}

export interface Adapter<Query, Resource> {
  buildQuery: (f: Filter) => Query;
  executeQuery: (q: Query) => Promise<Resource[]>;
}

export type Datum = Projection | Immediate;

export interface FilterCondition {
  lhs: Datum;
  cmp: PolarComparisonOperator;
  rhs: Datum;
}

export type FilterConditionSide = 'lhs' | 'rhs';

export interface Filter {
  model: string;
  relations: FilterRelation[];
  conditions: FilterCondition[][];
  types: { [tag: string]: SerializedFields };
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
    types: host.serializeTypes(),
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
