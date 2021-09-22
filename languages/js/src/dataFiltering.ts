import { Host, UserTypesMap } from './Host';
import { isPolarTerm, obj } from './types';
import { isObj } from './helpers';

export interface SerializedRelation {
  Relation: {
    kind: string;
    other_class_tag: string;
    my_field: string;
    other_field: string;
  };
}

/** Represents relationships between two resources, eg. one-one or one-many. */
export class Relation {
  kind: string;
  otherType: string;
  myField: string;
  otherField: string;

  constructor(
    kind: string,
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

export class Field {
  field: string;

  constructor(field: string) {
    this.field = field;
  }
}

class Ref {
  field: string;
  resultId: string;

  constructor(field: string, resultId: string) {
    this.field = field;
    this.resultId = resultId;
  }
}

/** Represents a condition that must hold on a resource. */
export class Filter {
  kind: string;
  field: string;
  value: Ref | Field | unknown;

  constructor(kind: string, field: string, value: unknown) {
    this.kind = kind;
    this.field = field;
    this.value = value;
  }
}

type SerializedFields = {
  [field: string]: SerializedRelation | { Base: { class_tag: string } };
};

export function serializeTypes(userTypes: UserTypesMap): string {
  const polarTypes: { [tag: string]: SerializedFields } = {};
  for (const [tag, userType] of userTypes.entries())
    if (typeof tag === 'string') {
      const fields = userType.fields;
      const fieldTypes: SerializedFields = {};
      for (const [k, v] of fields.entries()) {
        if (v instanceof Relation) {
          fieldTypes[k] = v.serialize();
        } else {
          const class_tag = userTypes.get(v)?.name;
          // TODO(gj): what's the failure mode if `userType` is undefined?
          if (class_tag === undefined) throw new Error();
          fieldTypes[k] = { Base: { class_tag } };
        }
      }
      polarTypes[tag] = fieldTypes;
    }
  return JSON.stringify(polarTypes);
}

async function parseFilter(host: Host, filter: obj): Promise<Filter> {
  const { kind, field } = filter;
  if (typeof kind !== 'string') throw new Error();
  if (typeof field !== 'string') throw new Error();

  let { value } = filter;
  if (!isObj(value)) throw new Error();

  if (isPolarTerm(value['Term'])) {
    value = await host.toJs(value['Term']);
  } else if (value['Ref'] !== undefined) {
    const { field: childField, result_id: resultId } = value;
    if (typeof childField !== 'string') throw new Error();
    if (typeof resultId !== 'string') throw new Error();
    value = new Ref(childField, resultId);
  } else if (typeof value['Field'] === 'string') {
    value = new Field(value['Field']);
  } else {
    throw new Error();
  }

  return new Filter(kind, field, value);
}

function groundFilter(results: any, filter: Filter) {
  const ref = filter.value;
  if (!(ref instanceof Ref)) return;
  filter.value = results.get(ref.resultId);
  // TODO(gj): can `ref.field` ever be anything but a string? If it can't, is
  // this condition just checking that it's non-empty?
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (ref.field) filter.value = filter.value.map((v: obj) => v[ref.field]);
}

// @TODO: type for filter plan

export async function filterData(host: Host, plan: any): Promise<any> {
  const queries: any = [];
  let combine: any;
  for (const rs of plan.result_sets) {
    const setResults = new Map();
    for (const i of rs.resolve_order) {
      const req = rs.requests.get(i);
      const constraints = req.constraints;

      for (const i in constraints) {
        const con = await parseFilter(host, constraints[i]);
        // Substitute in results from previous requests.
        groundFilter(setResults, con);
        constraints[i] = con;
      }

      const typ = host.getType(req.class_tag)!;
      const query = await Promise.resolve(typ.buildQuery!(constraints));
      if (i !== rs.result_id) {
        setResults.set(i, await Promise.resolve(typ.execQuery!(query)));
      } else {
        queries.push(query);
        combine = typ.combineQuery!;
      }
    }
  }

  if (queries.length === 0) return null;
  // @TODO remove duplicates
  return queries.reduce(combine);
}
