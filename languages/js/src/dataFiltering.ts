import { resolve } from 'path/posix';
import { resourceUsage } from 'process';
import { Host, UserType } from './Host';

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
  let polarTypes: any = {};
  for (let [tag, userType] of userTypes.entries())
    if (typeof tag === 'string') {
      let fields = userType.fields;
      let fieldTypes: any = {};
      for (let [k, v] of fields.entries()) {
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
  let kind = constraint['kind'];
  let field = constraint['field'];
  let value = constraint['value'];

  let valueKind = Object.keys(value)[0];
  value = value[valueKind];
  if (valueKind == 'Term') {
    value = await host.toJs(value);
  } else if (valueKind == 'Ref') {
    let childField = value['field'];
    let resultId = value['result_id'];
    value = new Ref(childField, resultId);
  } else if (valueKind == 'Field') {
    value = new Field(value);
  }

  return new Filter(kind, field, value);
}

function groundFilter(results: any, con: Filter) {
  let ref = con.value;
  if (!(ref instanceof Ref)) return;
  con.value = results.get(ref.resultId);
  if (ref.field != null) con.value = con.value.map((v: any) => v[ref.field]);
}

function groundFilters(results: any, constraints: any): any {
  for (let c of constraints) groundFilter(results, c);
  return constraints;
}

// @TODO: type for filter plan

export async function filterData(host: Host, plan: any): Promise<any> {
  let queries: any = [];
  let combine: any;
  for (let rs of plan.result_sets) {
    let setResults = new Map();

    for (let i of rs.resolve_order) {
      let req = rs.requests.get(i);
      let constraints = req.constraints;

      for (let i in constraints) {
        let con = await parseFilter(host, constraints[i]);
        // Substitute in results from previous requests.
        groundFilter(setResults, con);
        constraints[i] = con;
      }

      let typ = host.types.get(req.class_tag)!;
      let query = await Promise.resolve(typ.buildQuery!(constraints));
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
