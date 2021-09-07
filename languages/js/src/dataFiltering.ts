import { resolve } from 'path/posix';
import { resourceUsage } from 'process';
import { Host } from './Host';

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

class Field {
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

export class Constraint {
  kind: string;
  field: string;
  value: any;

  constructor(kind: string, field: string, value: any) {
    this.kind = kind;
    this.field = field;
    this.value = value;
  }
}

export function serializeTypes(
  types: Map<string, any>,
  clsNames: Map<any, string>
): string {
  let polarTypes: any = {};
  for (let [tag, fields] of types.entries()) {
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
            class_tag: clsNames.get(v),
          },
        };
      }
    }
    polarTypes[tag] = fieldTypes;
  }
  return JSON.stringify(polarTypes);
}

async function parseConstraint(
  host: Host,
  constraint: any
): Promise<Constraint> {
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

  return new Constraint(kind, field, value);
}

function groundConstraints(
  host: Host,
  results: any,
  plan: any,
  constraints: any
): any {
  for (let i in constraints) {
    if (constraints[i].value instanceof Ref) {
      let ref = constraints[i].value;
      constraints[i].value = results.get(ref.resultId)!;
      if (ref.field != null) {
        for (let j in constraints[i].value) {
          constraints[i].value[j] = constraints[i].value[j][ref.field];
        }
      }
    }
  }
  return constraints;
}

// @TODO: type for filter plan

export async function filterData(host: Host, plan: any): Promise<any> {
  let resultSets = plan['result_sets'];
  let results: any = [];
  for (let rs of resultSets) {
    let setResults = new Map();
    let requests = rs['requests'];
    let resolveOrder = rs['resolve_order'];
    let resultId = rs['result_id'];

    for (let i of resolveOrder) {
      let req = requests.get(i);
      let className = req['class_tag'];
      let constraints = req['constraints'];

      for (let i in constraints) {
        constraints[i] = await parseConstraint(host, constraints[i]);
      }

      // Substitute in results from previous requests.
      constraints = groundConstraints(host, setResults, plan, constraints);
      let fetcher = host.fetchers.get(className);
      let fetched = fetcher(constraints);
      fetched = await Promise.resolve(fetched);
      setResults.set(i, fetched);
    }
    results = results.concat(setResults.get(resultId)!);
  }
  // @TODO remove duplicates
  return results;
}
