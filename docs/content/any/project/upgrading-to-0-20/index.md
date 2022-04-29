---
title: Upgrading to 0.20
weight: 2
any: false
description: |
  Migration guide for upgrading from Oso 0.15 to the new 0.20 release.
---

# Upgrading to 0.20

Oso's [0.20 release](project/changelogs/2021-09-15) is larger than usual as it
includes a number of new features around authorization [modeling](guides/rbac),
[enforcement](guides/enforcement), and [data filtering](guides/data_filtering).

To migrate to 0.20 from 0.15, the previous stable release, follow the steps in
this guide.

## Parenthesize `or` operations

The [`or` operator](polar-syntax#disjunction-or) has had its precedence lowered
to be consistent with other programming languages. Existing policies using `or`
should be updated where necessary to group `or` operations using parentheses:

To migrate, rewrite:

```polar
foo(a, b, c) if a and b or c;
```

as:

```polar
foo(a, b, c) if a and (b or c);
```

We have temporarily made policies that combine `and` and `or` _without_
parentheses raise warnings in order to avoid silent changes. To silence the
warning, add parentheses.

## Consolidate `{{% exampleGet "load_file" %}}()` calls into a single `{{% exampleGet "load_file" %}}s()` call

As of 0.20, all Polar policies must be loaded in one fell swoop. To facilitate
this,
{{% apiDeepLink class="Oso" %}}{{% exampleGet "load_file" %}}{{% /apiDeepLink %}}
has been deprecated and replaced with
{{% apiDeepLink class="Oso" %}}{{% exampleGet "load_file" %}}s{{% /apiDeepLink %}},
which loads multiple Polar policy files at once.

{{% callout "Warning" "orange" %}}
  Calling `{{% exampleGet "load_file" %}}()`,
  `{{% exampleGet "load_file" %}}s()`, or `{{% exampleGet "load_str" %}}()`
  more than once is now an error.
{{% /callout %}}

Continued use of `{{% exampleGet "load_file" %}}()` will result in deprecation
warnings printed to the console. **`{{% exampleGet "load_file" %}}()` will be
removed in a future release.**

To migrate, rewrite:

```{{% lang %}}
oso.{{% exampleGet "load_file" %}}("main.polar")
oso.{{% exampleGet "load_file" %}}("repository.polar")
```

as:

```{{% lang %}}
oso.{{% exampleGet "load_file" %}}s({{% exampleGet "files" %}})
```

{{% ifLang "node" %}}
  ## `registerClass()` API change

  The second argument to `registerClass()` has been changed from a single
  optional string to an object or Map to accommodate additional optional
  parameters.

  If you weren't using the second argument, there's no change required:

  ```js
  // Previously:
  oso.registerClass(Foo);
  // Still:
  oso.registerClass(Foo);
  ```

  If you were passing a name as the second argument, lift it into the `name` key
  of a JavaScript object:

  ```js
  // Previously:
  oso.registerClass(Foo, 'Bar');
  // Now:
  oso.registerClass(Foo, { name: 'Bar' });
  ```
{{% /ifLang %}}

{{% ifLang "python" %}}
  ## Migrating from Polar Roles or SQLAlchemy Roles to the new resource block syntax
{{% /ifLang %}}
{{% ifLang not="python" %}}
  ## Migrating from Polar Roles to the new resource block syntax
{{% /ifLang %}}

The 0.20 release introduces [resource
blocks](reference/polar/polar-syntax#actor-and-resource-blocks) for expressing
role-based (RBAC) and relationship-based (ReBAC) access control logic in a
concise, readable syntax.

For a taste of the new resource block syntax, see the [Build Role-Based Access
Control (RBAC)](/guides/rbac) guide.

{{% ifLang "python" %}}
  The previous Polar Roles and SQLAlchemy Roles features have been removed, and
  we encourage all users to upgrade to the new syntax, which supports a
  superset of the functionality of the previous feature.
{{% /ifLang %}}
{{% ifLang not="python" %}}
  The previous Polar Roles feature has been removed, and we encourage all users
  to upgrade to the new syntax, which supports a superset of the functionality
  of the previous feature.
{{% /ifLang %}}

{{% ifLang "python" %}}
  **If you are using Polar Roles or SQLAlchemy Roles with a previous version of
  Oso and want to upgrade to the 0.20 release, please reach out via
  [Slack](https://join-slack.osohq.com/) or [schedule a 1-on-1 with an engineer
  from the core team](https://calendly.com/osohq/1-on-1?utm_source=library_docs&utm_content=upgrade_to_020).**
{{% /ifLang %}}
{{% ifLang not="python" %}}
  **If you are using Polar Roles with a previous version of Oso and want to
  upgrade to the 0.20 release, please reach out via
  [Slack](https://join-slack.osohq.com/) or [schedule a 1-on-1 with an engineer
  from the core team](https://calendly.com/osohq/1-on-1?utm_source=library_docs&utm_content=upgrade_to_020).**
{{% /ifLang %}}

{{% ifLang not="rust" %}}
  ## Migrate from `{{% exampleGet "is_allowed" %}}()` to `{{% exampleGet "authorize" %}}()`

  The 0.20 release introduces
  {{% apiDeepLink class="Oso" %}}{{% exampleGet "authorize" %}}{{% /apiDeepLink %}},
  a new method that supplants
  {{% apiDeepLink class="Oso" %}}{{% exampleGet "is_allowed" %}}{{% /apiDeepLink %}}
  as the preferred API for [resource-level
  enforcement](guides/enforcement/resource).

  The methods' return values are different: `{{% exampleGet "is_allowed" %}}()`
  returns a boolean and expects you to translate `{{% exampleGet "false" %}}`
  into an authorization failure, but `{{% exampleGet "authorize" %}}()` raises
  an Oso authorization error, which you can translate into your own error type.

  To migrate, rewrite:

  {{% exampleGet "authorize_migration" %}}
{{% /ifLang %}}
