
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
relationship(o: Organization, r: Repository, role_map) if
    o = r.org and
    # map from org to repo roles
    role_map = {
        owner: "write",
        member: "read"
    };

