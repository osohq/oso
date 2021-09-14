require 'oso'

Relation = Oso::Polar::DataFiltering::Relation

oso = Oso.new

# Register the Organization class
oso.register_class(
  Organization,
  fields: { id: String }
)

# Register the Repository class, and its relation to the Organization class
oso.register_class(
  Repository,
  fields: {
    id: String,
    organization: Relation.new(
      kind: 'one',
      other_type: Organization,
      my_field: 'org_id',
      other_field: 'id'
    )
  }
)
