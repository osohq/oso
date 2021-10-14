from oso import Oso, Relation

oso = Oso()

# Register the Organization class
oso.register_class(
    Organization,
    types={
        "id": str,
    },
    build_query=build_query_cls(Organization),
)

# Register the Repository class, and its relation to the Organization class
oso.register_class(
    Repository,
    types={
        "id": str,
        "organization": Relation(
            kind="one", other_type="Organization", my_field="org_id", other_field="id"
        ),
    },
    build_query=build_query_cls(Repository),
)
