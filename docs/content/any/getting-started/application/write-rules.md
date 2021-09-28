---
title: Write Polar Rules
description: |
    Polar is Oso's declarative policy language. To use Oso, you model
    your authorization logic as a set of rules.In this guide, we'll go
    into more detail on writing rules yourself.
weight: 3
---

# Write Polar Rules

Polar is Oso's declarative policy language. We used Polar in [Model your
authorization logic](model) to authorize access to resources, and we
already wrote a few rules:

- an `allow` rule: The top level rule that `authorize` calls to perform
  resource-level enforcement.
- a `has_role` rule: The rule used to tell Oso whether an actor has a
  particular role on a resource

In this guide, we'll go into more detail on writing rules yourself.

## What's in a rule

To use Oso, you model your authorization logic as a set of rules. This
set of rules is called the *policy*. Let's take a look at a basic rule:

```polar
allow(actor, action, resource) if
	has_permission(actor, action, resource);
```

The rule has a *name*, and *parameter list*. The name is `allow`, and
the parameters are `actor`, `action`, and `resource` (all variables).
This rule has a body that calls another rule, `has_permission` with the
same set of parameters. This `allow` rule will succeed if
`has_permission` succeeds.

A rule succeeds if the parameters it's called with *match* its parameter
list and if the conditions in the body of the rule succeed.

## Rules with conditions

You can write conditions in the body of a rule. For example:

```polar
has_permission(_user: User, "read", repository: Repository) if
	repository.is_public = true;
```

This rule succeeds if the `repository` 's `is_public` property is equal
to `true` and the parameter list matches.

In this rule, we used types on our parameters, and specified a literal
as the `action` parameter, meaning the rule will only match if the
action is `"read"`.

## Combining conditions

You can specify multiple conditions in a rule body using the `and`
operator:

```polar
allow(user, action, resource) if
	user.is_blocked = false and
	has_permission(user, action, resource);
```

This rule succeeds if the `user` `is_blocked` field is `false` *and*
the `has_permission` rule succeeds.

## Extending resource blocks

We defined *shorthand* *rules* in our *resource blocks* in [Model your
authorization policy](model). Let's take a look at extending these
rules:

```polar
resource Repository {
    permissions = ["read", "push", "delete"];
    roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
    "push" if "maintainer";
    "delete" if "admin";

    "maintainer" if "admin";
    "contributor" if "maintainer";
}
```

Each *shorthand rule* expands to a rule when the *resource block* is
parsed. For example, `"read" if "contributor"` expands to:

```polar
has_permission(user: User, "read", repository: Repository) if
	has_role(user, "contributor", repository);
```

We can grant users permissions based on roles by writing our own rules:

```polar
has_permission(user: User, "delete", repository: Repository) if
	# User has the "admin" role.
	has_role(user, "admin", repository) and
	user.auth_token.has_sudo_mode() = true;
```

This rule succeeds if the user has an `"admin"` role, and the
`has_sudo_mode()` method on the `auth_token` field of `user` returns
`true`. That's right! You can call {{% lang %}} methods directly from Polar,
and access their return values. This can help you share business logic
between the policy and your application, for example whether a user has
elevated their privileges sufficiently to perform dangerous actions.

## What's next

- For more detail on writing policies, see the [Write Oso Policies](/guides/policies) guide.
- To go deeper on any of the concepts from this getting started section,
  see our [How to guides](/guides).
- To see how to apply authorization to large collections of data that
  cannot be loaded into memory, read [Filter collections of
  data](filter-data).
