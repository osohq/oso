# ALLOW RULES

### Allow superusers to view everything
allow(user: github::User, _action, _resource) if
    user.is_staff;


# RBAC BASE POLICY

## Top-level RBAC allow rule

### To disable the RBAC policy, simply comment out this rule
allow(user: github::User, action: String, resource) if
    rbac_allow(user, action, resource);

### The association between the resource roles and the requested resource is outsourced from the rbac_allow
rbac_allow(actor: github::User, action, resource) if
    resource_role_applies_to(resource, role_resource) and
    user_in_role(actor, role, role_resource) and
    role_allow(role, action, resource);

## Resource-role relationships

### These rules allow a roles to apply to resources other than those that they are scoped to.
### The most common example of this is nested resources, e.g. Repository roles should apply to the Issues
### nested in that repository.

### A resource's roles applies to itself
resource_role_applies_to(role_resource, role_resource);

### A repository's roles apply to its child resources (issues)
resource_role_applies_to(issue: github::Issue, parent_repo) if
    parent_repo = issue.repository;

### An organization's roles apply to its child resources (repos, teams)
resource_role_applies_to(repo: github::Repository, parent_org) if
    parent_org = repo.organization;

resource_role_applies_to(team: github::Team, parent_org) if
    parent_org = team.organization;


### Org roles apply to HttpRequests with paths starting /orgs/<org_name>/
resource_role_applies_to(requested_resource: HttpRequest, role_resource) if
    requested_resource.path.split("/") matches ["", "orgs", org_name, *_rest] and
    role_resource = github::Organization.objects.get(name: org_name);

### Repo roles apply to HttpRequests with paths starting /orgs/<org_name>/repos/<repo_name>/
resource_role_applies_to(requested_resource: HttpRequest, role_resource) if
    requested_resource.path.split("/") matches ["", "orgs", _org_name, "repos", repo_name, *_rest] and
    repo_name != "" and
    role_resource = github::Repository.objects.get(name: repo_name);


# USER-ROLE RELATIONSHIPS

## Organization Roles

### User role source: direct mapping between users and organization roles
user_in_role(user: github::User, role, org: github::Organization) if
    # role is an OrganizationRole object
    role in github::OrganizationRole.objects.filter(users: user) and
    role.organization.id = org.id;


## Organization Helpers

### User in organization
user_in_org(user: github::User, org: github::Organization) if
    user_in_role(user, role, org) and
    # check that role came back bound, so the user has at least one role on the org
    role matches github::OrganizationRole;


## Team Roles

### User role source: direct mapping between users and team roles
user_in_role(user: github::User, role, team: github::Team) if
    role in github::TeamRole.objects.filter(users: user) and
    role.team.id = team.id;

## Team Helpers

### Get user's teams
get_user_teams(user: github::User, team) if
    team in github::Team.objects.filter(teamrole__users: user);

## Repository Roles

### User role source: direct mapping between users and repository roles
user_in_role(user: github::User, role, repo: github::Repository) if
    # role is a RepositoryRole object
    role in github::RepositoryRole.objects.filter(users: user) and
    role.repository.id = repo.id;


### Team role source: direct mapping between teams and repository roles
team_in_role(team: github::Team, role, repo: github::Repository) if
    role in github::RepositoryRole.objects.filter(teams: team) and
    role.repository.id = repo.id;

### User role source: team role
user_in_role(user: github::User, role, repo: github::Repository) if
    get_user_teams(user, team) and
    team_in_role(team, role, repo);


# ROLE-PERMISSION RELATIONSHIPS

## Record-level Organization Permissions

### All organization roles let users read organizations
role_allow(role: github::OrganizationRole, "read", org: github::Organization) if
    role.organization.id = org.id;

## Route-level Organization Permissions

### Organization owners can access the "People" org page
role_allow(role: github::OrganizationRole{name: "Owner"}, "GET", request: HttpRequest) if
    request.path.split("/") matches ["", "orgs", org_name, "people", ""] and
    # this is enforced in the `rbac_allow` rule, but checking here to be safe
    role.organization.name = org_name;

### Organization members can access the "Teams" and "Repositories" pages within their organizations
role_allow(role: github::OrganizationRole{name: "Member"}, "GET", request: HttpRequest) if
    request.path.split("/") matches ["", "orgs", org_name, page, ""] and
    page in ["teams", "repos"] and
    role.organization.name = org_name;

### Organization members can hit the route to create repositories
role_allow(role: github::OrganizationRole{name: "Member"}, "POST", request: HttpRequest) if
    request.path.split("/") matches ["", "orgs", org_name, "repos", ""] and
    role.organization.name = org_name;

## Repository Permissions

### Read role can read the repository
role_allow(role: github::RepositoryRole{name: "Read"}, "read", repo: github::Repository) if
    role.repository.id = repo.id;

### Read role can read the repository's issues
role_allow(role: github::RepositoryRole{name: "Read"}, "read", issue: github::Issue) if
    role.repository.id = issue.repository.id;

### Organization "Read" base roles
role_allow(role: github::OrganizationRole{name: "Member"}, "read", repo: github::Repository) if
    role.organization = repo.organization and
    repo.organization.base_role = "Read";

### Repository admins can access the "Roles" repo page
role_allow(role: github::RepositoryRole{name: "Admin"}, _action, request: HttpRequest) if
    request.path.split("/") matches ["", "orgs", _org_name, "repos", repo_name, "roles", ""] and
    role.repository.name = repo_name;

### Organization owners can access the "Roles" repo page for all repos in the org
role_allow(role: github::OrganizationRole{name: "Owner"}, _action, request: HttpRequest) if
    request.path.split("/") matches ["", "orgs", org_name, "repos", _repo_name, "roles", ""] and
    role.organization.name = org_name;

### Repo Admins can take any action on their repositories
role_allow(role: github::RepositoryRole{name: "Admin"}, _action, repo: github::Repository) if
    role.repository.id = repo.id;

### Org Owners can take any action on their repositories
role_allow(role: github::OrganizationRole{name: "Owner"}, _action, repo: github::Repository) if
    role.organization.id = repo.organization.id;


## Team Permissions

### Organization owners can view all teams in the org
role_allow(role: github::OrganizationRole{name: "Owner"}, "read", team: github::Team) if
    role.organization = team.organization;

### Team members are able to see their own teams
role_allow(role: github::TeamRole{name: "Member"}, "read", team: github::Team) if
    role.team = team;

# ROLE-ROLE RELATIONSHIPS

## Role Hierarchies

### Grant a role permissions that it inherits from a more junior role
role_allow(role, action, resource) if
    inherits_role(role, inherited_role) and
    role_allow(inherited_role, action, resource);


### Role inheritance for repository roles
inherits_role(role: github::RepositoryRole, inherited_role) if
    repository_role_order(role_order) and
    inherits_role_helper(role.name, inherited_role_name, role_order) and
    inherited_role = new github::RepositoryRole(name: inherited_role_name, repository: role.repository);

### Specify repository role order (most senior on left)
repository_role_order(["Admin", "Maintain", "Write", "Triage", "Read"]);


### Role inheritance for organization roles
inherits_role(role: github::OrganizationRole, inherited_role) if
    organization_role_order(role_order) and
    inherits_role_helper(role.name, inherited_role_name, role_order) and
    inherited_role = new github::OrganizationRole(name: inherited_role_name, organization: role.organization);

### Specify organization role order (most senior on left)
organization_role_order(["Owner", "Member"]);
organization_role_order(["Owner", "Billing"]);

### Role inheritance for team roles
inherits_role(role: github::TeamRole, inherited_role) if
    team_role_order(role_order) and
    inherits_role_helper(role.name, inherited_role_name, role_order) and
    inherited_role := new github::TeamRole(name: inherited_role_name, team: role.team);

### Specify team role order (most senior on left)
team_role_order(["Maintainer", "Member"]);

### Helper to determine relative order or roles in a list
inherits_role_helper(role, inherited_role, role_order) if
    ([first, *rest] = role_order and
    role = first and
    inherited_role in rest) or
    ([first, *rest] = role_order and
    inherits_role_helper(role, inherited_role, rest));
