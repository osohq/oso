relationship(subject, predicate, object) if
  implies(implier, predicate) and
  relationship(subject, implier, object);

relationship(subject, predicate, object) if
  implies(on(subject_predicate, object_predicate), predicate) and
  relationship(intermediate, object_predicate, object) and
  relationship(subject, subject_predicate, intermediate);

################################################################################

allow(actor, action, resource) if
  relationship(actor, action, resource);

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

implies(on(repo_role("writer"), "parent"), "edit");
implies(on(repo_role("admin"), "parent"), "delete");

# role-role implications
implies(org_role("owner"), org_role("member"));

implies(on(org_role("owner"), "parent"), repo_role("admin"));
implies(on(org_role("member"), "parent"), repo_role("reader"));

implies(repo_role("admin"), repo_role("writer"));
implies(repo_role("writer"), repo_role("reader"));

relationship(org, "parent", repo: Repo) if
  org = repo.org and
  org matches Org; # redundant check in non-partial world since we already know repo.org is an Org

relationship(repo, "parent", issue: Issue) if
  repo = issue.repo and
  repo matches Repo; # redundant check in non-partial world since we already know issue.repo is a Repo

relationship(user: User, "owns", issue: Issue) if
  user = issue.created_by;
relationship(user: User, "owns", org: Org) if
  user = org.owner;

# BURRATA permission implication
implies("owns", "delete");

# BURRATA role implication
implies(on("owns", "parent"), repo_role("admin"));
