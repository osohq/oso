import { Host } from './Host';
import { obj, isPolarTerm } from './types';
import type { CombineQueryFn } from './types';
import { isObj, isString } from './helpers';
import { UnregisteredClassError } from './errors';

interface Request {
  class_tag: string;
  constraints: Filter[];
}

interface ResultSet {
  result_id: number;
  resolve_order: number[];
  requests: Map<number, Request>;
}

export interface FilterPlan {
  result_sets: ResultSet[];
}

interface SerializedRelation {
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

export class Field {
  field: string;

  constructor(field: string) {
    this.field = field;
  }
}

class Ref {
  resultId: number;
  field?: string;

  constructor(resultId: number, field?: string) {
    this.resultId = resultId;
    this.field = field;
  }
}

export type FilterKind = 'Eq' | 'Neq' | 'In' | 'Contains' | 'Nin';

export type FilterField = string | undefined | FilterField[];

/** Represents a condition that must hold on a resource. */
export interface Filter {
  kind: FilterKind;
  value: unknown; // Ref | Field | Term
  field: FilterField;
}

export type SerializedFields = {
  [field: string]: SerializedRelation | { Base: { class_tag: string } };
};

async function parseFilter(host: Host, filter: Filter): Promise<Filter> {
  const { kind, field } = filter;
  if (!['Eq', 'Neq', 'In', 'Contains', 'Nin'].includes(kind)) throw new Error();
  if (field !== undefined && !isString(field)) throw new Error();

  let { value } = filter;
  if (!isObj(value)) throw new Error();

  if (isPolarTerm(value['Term'])) {
    value = await host.toJs(value['Term']);
  } else if (isObj(value['Ref'])) {
    const { field: childField, result_id: resultId } = value['Ref'];
    if (childField !== undefined && !isString(childField)) throw new Error();
    if (!Number.isInteger(resultId)) throw new Error();
    value = new Ref(resultId as number, childField);
  } else if (isString(value['Field'])) {
    value = new Field(value['Field']);
  } else {
    throw new Error();
  }

  return { kind, value, field };
}

type SetResults = Map<number, unknown[]>;

function partition<A>(coll: A[], pred: (a: A) => boolean): A[][] {
  const yes: A[] = [],
    no: A[] = [];
  for (const a of coll) (pred(a) ? yes : no).push(a);
  return [yes, no];
}

function groupBy<A, B>(coll: A[], fn: (a: A) => B): Map<B, A[]> {
  const map: Map<B, A[]> = new Map();
  for (const a of coll) {
    const key = fn(a);
    const maybe = map.get(key);
    if (maybe) maybe.push(a);
    else map.set(key, [a]);
  }
  return map;
}

function getattr(x: obj, attr: string | undefined): unknown {
  return attr ? x[attr] : x;
}

function groundFilters(results: SetResults, filters: Filter[]): Filter[] {
  const [refs, rest] = partition(filters, f => f.value instanceof Ref);
  const [yrefs, nrefs] = partition(
    refs,
    f => f.kind === 'In' || f.kind === 'Eq'
  );

  for (const { refs, kind } of [
    { refs: yrefs, kind: 'In' },
    { refs: nrefs, kind: 'Nin' },
  ])
    for (const [rid, fils] of groupBy(refs, f => (f.value as Ref).resultId))
      rest.push({
        kind: kind as FilterKind,
        field: fils.map(f => f.field),
        value: results // eslint-disable-line @typescript-eslint/no-non-null-assertion
          .get(rid)!
          .map(r => fils.map(f => getattr(r as obj, (f.value as Ref).field))),
      });
  return rest;
}

export async function filterData<T>(
  host: Host,
  plan: FilterPlan
): Promise<T | null> {
  const queries = [];
  let combine: CombineQueryFn<T> | undefined;
  for (const rs of plan.result_sets) {
    const setResults: Map<number, unknown[]> = new Map();
    for (const i of rs.resolve_order) {
      const req = rs.requests.get(i);
      if (req === undefined) throw new Error();

      const filters = groundFilters(
        setResults,
        await Promise.all(
          req.constraints.map(async f => await parseFilter(host, f))
        )
      );

      // NOTE(gj|gw): The class_tag on the request comes from serializeTypes(),
      // a function we use to pass type information to the core in order to
      // generate the filter plan. The type information is derived from
      // Host.userTypes, so anything you get back as a class_tag will exist as
      // a key in the Host.userTypes Map.
      const typ = host.getType(req.class_tag);
      if (typ === undefined) throw new UnregisteredClassError(req.class_tag);

      const query = await typ.buildQuery(filters); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
      if (i !== rs.result_id) {
        setResults.set(i, await typ.execQuery(query));
      } else {
        queries.push(query);
        combine = typ.combineQuery;
      }
    }
  }

  if (queries.length === 0) return null;

  // NOTE(gw|gj): combine will only be undefined in two cases: (1) if
  // result_set.result_id is not a member of result_set.resolve_order; (2)
  // there are no result_sets. Either one of these would be a bug in the data
  // filtering logic in the core.
  if (combine === undefined) throw new Error();

  // @TODO remove duplicates
  return queries.reduce(combine); // eslint-disable-line @typescript-eslint/no-unsafe-return
}
