import { Relation, Oso } from "oso";

const oso = new Oso();

oso.setDataFilteringQueryDefaults({ combineQuery, buildQuery });

// Register the Organization type
oso.registerClass(Organization, {
  execQuery: execFromRepo(Organization),
  types: {
    id: String,
  }
});

// Register the Repository class, and its relation to the Organization type
oso.registerClass(Repository, {
  execQuery: execFromRepo(Repository),
  types: {
    id: String,
    organization: new Relation("one", "Organization", "org_id", "id"),
  }
});
