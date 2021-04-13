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

Now, Leina can create a repo becuase she's an owner, and inherits the privileges of members:

```python
assert oso.is_allowed(leina, "create_repo", osohq)
```

# 3. Resource Relationships

So far, our roles only grant permissions that can be taken on resources of type `Organization`.
