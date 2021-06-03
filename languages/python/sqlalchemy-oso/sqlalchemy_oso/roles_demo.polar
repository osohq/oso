
# role(resource, definitions, implies)


# organization role
role(Organization, definitions, implies) if
    definitions = {
        owner: ["invite"],
        member: ["create_repo"]
    } and
    implies = {
        owner: "member"
    };

# repository role
role(Repository, definitions, implies) if
    definitions = {
        write: ["push"],
        read: ["pull"]
    } and
    implies = {
        write: "read"
    };

# relationship(parent, child, role_map) if <condition>
relationship(o: Organization, r, role_map) if
    #### TODO: This should be o = r.organization
    #### But this constraint isn't being grounded/evaluated appropriately
    #### If I do this differently
    r in o.repositories and
    r matches Repository and
    # map from org to repo roles
    role_map = {
        owner: "write",
        member: "read"
    };

