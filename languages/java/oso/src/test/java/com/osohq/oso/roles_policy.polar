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

parent_child(parent_org, repo: Repo) if
    repo.org = parent_org and
    parent_org matches Org;

parent_child(parent_repo, issue: Issue) if
    issue.repo = parent_repo and
    parent_repo matches Repo;

actor_has_role_for_resource(actor, role_name, role_resource) if
    role in actor.roles and
    role matches {name: role_name, resource: role_resource};

allow(actor, action, resource) if
    role_allows(actor, action, resource);
