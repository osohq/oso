import { Host } from './Host';
import {
  obj,
  isPolarTerm,
  PolarComparisonOperator,
  PolarTerm,
  PolarValue,
} from './types';
import { isObj, isString } from './helpers';
import { OsoError, UnregisteredClassError } from './errors';
import { DefaultNamingStrategy } from 'typeorm';
import { Comparator } from 'lodash';

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
  value: any;
}

export interface Adapter<Q, R> {
  buildQuery: (f: Filter) => Promise<Q>;
  executeQuery: (q: Q) => Promise<R[]>;
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

export async function parseFilter(
  filter_json: any,
  host: Host
): Promise<Filter> {
  let filter = {
    model: filter_json.root,
    relations: [] as FilterRelation[],
    conditions: [] as FilterCondition[][],
    types: host.serializeTypes(),
  };

  for (let r of filter_json.relations) {
    let [from, field, to] = r;
    let rel: FilterRelation = {
      fromTypeName: from,
      fromFieldName: field,
      toTypeName: to,
    };
    filter.relations.push(rel);
  }

  async function parseDatum(d: any, host: Host): Promise<Datum> {
    let k = Object.getOwnPropertyNames(d)[0];
    switch (k) {
      case 'Field': {
        let [type, field] = d[k];
        return {
          typeName: type,
          fieldName: field,
        };
      }
      case 'Immediate': {
        let value: PolarValue = d[k];
        let term: PolarTerm = {
          value: value,
        };
        let jsValue = await host.toJs(term);
        return {
          value: jsValue,
        };
      }
      default: {
        throw new OsoError(`Invalid filter json.`);
      }
    }
  }

  for (let cs of filter_json.conditions) {
    let and_group: FilterCondition[] = [];
    for (let c of cs) {
      let [l, op, r] = c;
      let condition = {
        lhs: await parseDatum(l, host),
        cmp: op as PolarComparisonOperator,
        rhs: await parseDatum(r, host),
      };
      and_group.push(condition);
    }
    filter.conditions.push(and_group);
  }

  return filter;
}
