resource(_type: Org, "org", actions, roles) if
    actions = [
        "invite",
        "create_repo"
    ] and
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

resource(_type: Repo, "repo", actions, roles) if
    actions = [
        "push",
        "pull"
    ] and
    roles = {
        writer: {
            permissions: ["push", "issue:edit"],
            implies: ["reader"]
        },
        reader: {
            permissions: ["pull"]
        }
    };

resource(_type: Issue, "issue", actions, {}) if
    actions = [
        "edit"
    ];

parent_child(parent_org: Org, repo: Repo) if
    repo.org = parent_org;

parent_child(parent_repo: Repo, issue: Issue) if
    issue.repo = parent_repo;

actor_has_role_for_resource(actor, role_name, role_resource) if
    role in actor.roles and
    role matches {name: role_name, resource: role_resource};

allow(actor, action, resource) if
    role_allows(actor, action, resource);
