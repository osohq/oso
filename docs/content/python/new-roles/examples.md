---
title: Role modeling by example
weight: 3
description: >
    Let's see how to build common role models with SQLAlchemy roles.
---

# Role modeling by example

In this guide, we'll cover a few different role models in the context of
the [GitClub example application](https://github.com/osohq/gitclub-sqlalchemy-flask-react).

{{% callout "Read first" "blue" %}}
 - [Getting started](getting-started)
{{% /callout %}}

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

## Repository roles

## Cross resource implied roles

## 
