---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` NEW_VERSION

### Core

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

Data filtering now treats accessing an undeclared field as an error. All fields
used in data filtering queries must now be registered ahead of time by including
them in the `fields` parameter of the `register_class` Oso API function, which
previously was only necessary for data relations. For example, to use data filtering
with the rule

```
allow(user, _, foo: Foo) if foo.user_name = user.name;
```

`user_name` must be included in the `register_class` call for `Foo`

```
# an example in Python
oso.register_class(Foo, fields={'user_name': str })
```

This change *only* affects data filtering. Other Oso APIs require no new configuration.

#### Other bugs & improvements

- Fixed a bug where a negated constraint on a dot lookup could cause Polar to crash
  when the underlying variable became bound.
- Removed syntax for parenthesized specializers like `f(_: (x));`, which don't
  currently achieve anything.

### Python

#### Other bugs & improvements
- Thanks to [Clara McCreery](https://github.com/chmccreery) for a correction to our
  Python data filtering docs!

#### Platform support

We now publish wheels for musl-based Linux distributions (through the `musllinux`
tag), and for ARM-based MacOS systems (through the `macosx_11_0_arm64` tag).

On those systems, you should now be able to use `pip install oso` to get the
latest Oso package.

### Node.js

#### Other bugs & improvements
- The `Class` type for representing abstract resources for data filtering is
  now a top-level export.

### LANGUAGE (e.g., 'Core' or 'Python' or 'Node.js')

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().

## `sqlalchemy-oso` NEW_VERSION

### Support for authorization of eager loaded data

`sqlalchemy-oso` will now detect entities that are loaded due to
[relationship loading options]() declared on the model or query. For example:

```python
a = query(A).options(joinedload(A.bs)).all()
bs = a[0].bs
```

`bs` will now contain only authorized data according to the policy.
In previous versions of `sqlalchemy-oso`, `bs` would not be authorized with the
`joinedload` option.


[relationship loading options]: https://docs.sqlalchemy.org/en/14/orm/loading_relationships.html
