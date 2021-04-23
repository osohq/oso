---
title: Add Roles (NEW)
weight: 1
description: >
  New Oso roles demo!
---

# Add roles to your app with Oso (NEW)

## Your application data

Before adding roles to your app, you'll need classes that represent your application's **users** and the **resources** you want to control access to with roles. For example:

```python
@dataclass
class User:
    name: str

@dataclass
class Organization:
    id: str

@dataclass
class Repository:
    id: str
    org: Organization
```

## 1. Simple Organization roles

Let's begin by adding some built-in roles to our app at the organization level. This means that these roles will control access within a single organization, but won't provide fine-grained access control over resources within an organization (e.g., repository-level access control).

First, set up Oso:

```python
# Set up oso
oso = Oso()
oso.register_class(User)
oso.register_class(Organization)

# Set up roles
roles = OsoRoles(oso)
roles.enable()
```

Now, let's configure our organization roles in our Polar policy:

```polar
# Define Organization roles
resource(_type: Organization, "org", actions, roles) if
    actions = [     # the actions that exist for Organizations
        "invite",
        "create_repo"
    ] and
    roles = {       # the roles that exist for organizations
        org_member: {
            perms: ["create_repo"]  # role-permission assignments
        },
        org_owner: {
            perms: ["invite"]
        }
    };

# Use roles to evaluate allow queries
allow(actor, action, resource) if
    Roles.role_allows(actor, action, resource);
```

After loading the policy, we can assign users to roles:

```python
# Load the policy file
oso.load_str("policy.polar")

# Demo data
osohq = Organization(id="osohq")

leina = User(name="Leina")
steve = User(name="Steve")

# Assign users to roles
roles.assign_role(leina, osohq, "org_owner")
roles.assign_role(steve, osohq, "org_member")
```

Let's write a few tests to show that the roles are working:

```python
# Leina can invite people to osohq because she is an OWNER
assert oso.is_allowed(leina, "invite", osohq)

# Steve can create repos in osohq because he is a MEMBER
assert oso.is_allowed(steve, "create_repo", osohq)

# Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
assert not oso.is_allowed(steve, "invite", osohq)
```

Now, what if Leina tries to create a repo?

```python
assert oso.is_allowed(leina, "create_repo", osohq)
```

This `assert` failed, because even though Leina is the `org_owner`, we didn't assign her permissions to `create_repo`. We could assign it to her directly, but what we actually want to do is say that the `org_owner` role can do everything the `org_member` role can do.

We can actually do that with **implied roles**.

## 2. Implied roles

Implied roles allow us to specify that having `role_a` implies having `role_b`. Meaning if you're in `role_a`, you're automatically granted the permissions of `role_b` as well.

We can use this to say that the `org_owner` role implies the `org_member` role, to avoid repeating permission assignments across these roles.

We'll add this implied role relation to our role configuration in the policy file:

```polar
resource(_resource: Organization, "org", actions, roles) if
    actions = [
        "invite",
        "create_repo"
    ] and
    roles = {
        org_owner: {
            perms: ["invite"],
            implies: ["org_member"]     # org_owner implies org_member
        },
        org_member: {
            perms: ["create_repo"]
        }
    };

allow(actor, action, resource) if
    Roles.role_allows(actor, action, resource);
```

Now, Leina can create a repo because she's an owner, and inherits the privileges of members:

```python
assert oso.is_allowed(leina, "create_repo", osohq)
```

# 3. Resource Relationships

So far, our roles only grant permissions that can be taken on resources of type `Organization`.

But what if we want to control access to repositories inside the organization?
In our policy, let's define some repository permissions that we'd like to use to control access to repos:

```polar
resource(_resource: Repository, "repo", actions, _) if
    actions = [
        "push",
        "pull"
    ];
```

We'd like to let `org_members` pull from and push to all repos in the org.

In order to assign repo permissions to organization roles, we need to tell Oso how repos and orgs are related. We can do this by creating a resource **relation** in Oso. So far, the only type of relation supported is a **parent** relation, which is defined using the `parent` predicate in our policy file:

```polar
parent(repository: Repository, parent_org: Organization) if
    repository.org = parent_org;
```

Test:

```python
# Demo oso repo
oso_repo = Repository(id="oso", org=osohq)

# Steve can push and pull from repos in the osohq org because he is a member of the org
assert oso.is_allowed(steve, "pull", oso_repo)
assert oso.is_allowed(steve, "push", oso_repo)

# Leina can push and pull from repos in the osohq org because she is an owner of the org, and therefore has
# the same privileges as members of the org
assert oso.is_allowed(leina, "pull", oso_repo)
assert oso.is_allowed(leina, "push", oso_repo)
```

## 4. Repository Roles

Add roles for `Repository`:

```polar
resource(_type: Organization, "org", actions, roles) if
    actions = [
        "invite",
        "create_repo"
    ] and
    roles = {
        org_member: {
            perms: ["create_repo"] # remove repo permissions
        },
        org_owner: {
            perms: ["invite"],
            implies: ["org_member"]
        }
    };

resource(_type: Repository, "repo", actions, roles) if
    actions = [
        "push",
        "pull"
    ] and
    roles = {   # add repo roles
        repo_write: {
            perms: ["push"],
            implies: ["repo_read"]
        },
        repo_read: {
            perms: ["pull"]
        }
    };

parent(repository: Repository, parent_org: Organization) if
    repository.org = parent_org;

allow(actor, action, resource) if
    Roles.role_allows(actor, action, resource);
```

Assign roles:

```python
# Now we can assign Leina and Steve to roles on the repo directly
roles.assign_role(leina, oso_repo, "repo_write")
roles.assign_role(steve, oso_repo, "repo_read")
```

Leina and Steve retain the same permissions as before:

```python
assert oso.is_allowed(steve, "pull", oso_repo)
assert not oso.is_allowed(steve, "push", oso_repo)

assert oso.is_allowed(leina, "pull", oso_repo)
assert oso.is_allowed(leina, "push", oso_repo)
```

## 5. Cross-resource implied roles

Now we have Organization roles and Repository roles.

Currently, these aren't related to one another. But what if we want all Organization members to have a base Repository role for all repos in the org?

We can do this with the same implied roles concept, where `org_member` implies `repo_read`, so that all organization members can pull from all repos in the org. This is a **cross-resource implied role**. Like assigning permissions across resource roles, adding a cross-resource implied role requires a `parent` relationship.

```polar
resource(_type: Organization, "org", actions, roles) if
    actions = [
        "invite",
        "create_repo"
    ] and
    roles = {
        org_member: {
            perms: ["create_repo"],
            implies: ["repo_read"]      # org_member implies repo_read
        },
        org_owner: {
            perms: ["invite"],
            implies: ["org_member", "repo_write"]   # org_owner implies repo_write
        }
    };
```

To show how this works, let's update our demo users and role assignments:

```python
leina = User(name="Leina")
steve = User(name="Steve")
gabe = User(name="Gabe")

# Role assignments
roles.assign_role(leina, osohq, "org_owner")
roles.assign_role(steve, osohq, "org_member")

roles.assign_role(gabe, oso_repo, "repo_write")
```

Test:

```python
# Steve can pull from oso_repo because he is a MEMBER of osohq
# which implies READ on oso_repo
assert oso.is_allowed(steve, "pull", oso_repo)
# Leina can pull from oso_repo because she's an OWNER of osohq
# which implies WRITE on oso_repo
# which implies READ on oso_repo
assert oso.is_allowed(leina, "pull", oso_repo)
# Gabe can pull from oso_repo because he has WRTIE on oso_repo
# which implies READ on oso_repo
assert oso.is_allowed(gabe, "pull", oso_repo)

# Steve can NOT push to oso_repo because he is a MEMBER of osohq
# which implies READ on oso_repo but not WRITE
assert not oso.is_allowed(steve, "push", oso_repo)
# Leina can push to oso_repo because she's an OWNER of osohq
# which implies WRITE on oso_repo
assert oso.is_allowed(leina, "push", oso_repo)
# Gabe can push to oso_repo because he has WRTIE on oso_repo
assert oso.is_allowed(gabe, "push", oso_repo)
```

## 6. Dynamic role permissions

Let's say we want to allow users to customize the Organization member role on a per organization basis.

In GitHub, this looks like a checkbox that lets users toggle whether org members can create private repos.

You can do this with Oso roles!

First, let's create the permission for creating private repos:

```polar
resource(_type: Organization, "org", actions, roles) if
    actions = [
        "invite",
        "create_repo",
        "create_private_repo"   # add permission for creating private repos in an org
    ] and
    roles = {
        org_member: {
            perms: ["create_repo"],
            implies: ["repo_read"]
        },
        org_owner: {
            perms: ["invite"],
            implies: ["org_member", "repo_write"]
        }
    };

```

Demo data:

```python
# Let's add a new organization
slack = Organization(id="slack")

# Gabe secretly works for slack????
roles.assign_role(gabe, slack, "org_member")
```

Now the magic happens. Let's say someone in the Slack organization just checked "allow members to create private repos" in the GitHub UI.
To effect this change, we call Oso's `roles.add_scoped_role_permission` method.

The "scope" is the resource that the modification should be scoped to (in this case, the scope is the Slack org). In this case, the scope resource is the same type of resource as the resource of the role we are scoping (i.e., they are both `Organization`).

```python
# Add a scoped role permission
# Slack organization members are also allowed to create private repos
roles.add_scoped_role_permission(
    scope=slack,
    role_name="org_member", perm_name="org:create_private_repo",
)
```

Test:

```python
# Gabe can create private repos in Slack because he is a MEMBER
assert oso.is_allowed(gabe, "create_private_repo", slack)

# Leina can't create private repos in osohq because it doesn't have that permission
assert not oso.is_allowed(leina, "create_private_repo", osohq)
```
