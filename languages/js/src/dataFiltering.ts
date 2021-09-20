import { resolve } from 'path/posix';
import { resourceUsage } from 'process';
import { Host, UserType } from './Host';

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
  value: any;

  constructor(kind: string, field: string, value: any) {
    this.kind = kind;
    this.field = field;
    this.value = value;
  }
}

export function serializeTypes(userTypes: Map<any, UserType>): string {
  const polarTypes: any = {};
  for (const [tag, userType] of userTypes.entries())
    if (typeof tag === 'string') {
      const fields = userType.fields;
      const fieldTypes: any = {};
      for (const [k, v] of fields.entries()) {
        if (v instanceof Relation) {
          fieldTypes[k] = {
            Relation: {
              kind: v.kind,
              other_class_tag: v.otherType,
              my_field: v.myField,
              other_field: v.otherField,
            },
          };
        } else {
          fieldTypes[k] = {
            Base: {
              class_tag: userTypes.get(v)?.name,
            },
          };
        }
      }
      polarTypes[tag] = fieldTypes;
    }
  return JSON.stringify(polarTypes);
}

async function parseFilter(host: Host, constraint: any): Promise<Filter> {
  const kind = constraint['kind'];
  const field = constraint['field'];
  let value = constraint['value'];

  const valueKind = Object.keys(value)[0];
  value = value[valueKind];
  if (valueKind == 'Term') {
    value = await host.toJs(value);
  } else if (valueKind == 'Ref') {
    const childField = value['field'];
    const resultId = value['result_id'];
    value = new Ref(childField, resultId);
  } else if (valueKind == 'Field') {
    value = new Field(value);
  }

  return new Filter(kind, field, value);
}

function groundFilter(results: any, con: Filter) {
  const ref = con.value;
  if (!(ref instanceof Ref)) return;
  con.value = results.get(ref.resultId);
  if (ref.field) con.value = con.value.map((v: any) => v[ref.field]);
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

      const typ = host.types.get(req.class_tag)!;
      const query = await Promise.resolve(typ.buildQuery!(constraints));
      if (i != rs.result_id) {
        setResults.set(i, await Promise.resolve(typ.execQuery!(query)));
      } else {
        queries.push(query);
        combine = typ.combineQuery!;
      }
    }
  }

  if (queries.length == 0) return null;
  // @TODO remove duplicates
  return queries.reduce(combine);
}
