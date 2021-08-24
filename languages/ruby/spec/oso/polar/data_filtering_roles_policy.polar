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
            implies: ["repo:writer", "member"]
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

actor_has_role_for_resource(actor, role_name: String, resource) if
    role in actor.roles and
    role.resource_name = resource.name and
    role.role = role_name;

allow(actor, action, resource) if
    role_allows(actor, action, resource);
