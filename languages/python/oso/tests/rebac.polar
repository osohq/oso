## TODO
# - user in group (teams)
# - repo.is_public
# - to think about:
#   - distinction between what's a relationship/3 and implications
#     - between "application data-based relationships" and "abstract relationships"

relationship(subject, predicate, object) if
  implies(implier, predicate) and
  relationship(subject, implier, object);

relationship(subject, predicate, object) if
  implies(on(subject_predicate, object_predicate), predicate) and
  relationship(intermediate, object_predicate, object) and
  relationship(subject, subject_predicate, intermediate);

allow(actor, action, resource) if
  relationship(actor, action, resource);

################################################################################

# (user, role, resource) -- user role assignments
relationship(user: User, org_role(role), org: Org) if
  user.has_role_for_resource(role, org);
relationship(user: User, repo_role(role), repo: Repo) if
  user.has_role_for_resource(role, repo);

# role-permission implications
implies(org_role("owner"), "invite");
implies(org_role("member"), "create_repo");

implies(repo_role("writer"), "push");
implies(repo_role("reader"), "pull");

# cross-resource role-permission implications
implies(on(repo_role("writer"), "parent"), "edit"); # "writer" on repo implies "edit" on issue that is a child of repo
implies(on(repo_role("admin"), "parent"), "delete");
## with enums:
# implies(on(repo.roles.admin, "parent"), issue.actions.delete);
## with enums & infix:
# repo.roles.admin on parent implies issue.actions.delete;

# role-role implications
implies(org_role("owner"), org_role("member"));

# cross-resource role-role implications
implies(on(org_role("owner"), "parent"), repo_role("admin")); # "owner" on parent org implies "admin" on child repo
implies(on(org_role("member"), "parent"), repo_role("reader"));
## with enums:
# implies(on(org.roles.member, "parent"), repo.roles.reader);

implies(repo_role("admin"), repo_role("writer"));
implies(repo_role("writer"), repo_role("reader"));

# resource-resource relationship (parent-child)
relationship(org, "parent", repo: Repo) if
  org = repo.org and
  org matches Org;
relationship(repo, "parent", issue: Issue) if
  repo = issue.repo and
  repo matches Repo;

# user-resource attribute relationship (ABURRTAJR)
relationship(user: User, "owns", issue: Issue) if
  user = issue.created_by;
relationship(user: User, "owns", org: Org) if
  user = org.owner;

# user-resource attribute permission implication (ABURRTAJR)
implies("owns", "delete");

# user-resource attribute role implication (ABURRTAJR)
implies(on("owns", "parent"), repo_role("admin"));
