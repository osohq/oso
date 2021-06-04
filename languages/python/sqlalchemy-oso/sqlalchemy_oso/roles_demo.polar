resource(Organization, "org", actions, roles) if
    actions = ["create_repo", "invite"] and
    roles = {
        member: {
            permissions: ["create_repo"],
            implies: ["repo:reader"]
        },
        owner: {
            permissions: ["invite"],
            implies: ["member", "repo:writer"]
        }
    };

resource(Repository, "repo", actions, roles) if
    actions = ["pull", "push"] and
    roles = {
        writer: {
            permissions: ["push"],
            implies: ["reader"]
        },
        reader: {
            permissions: ["pull"]
        }
    };

parent(org: Organization, repo: Repository) if
    repo.organization = org;
