---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `RELEASED_PACKAGE_1` NEW_VERSION

### Go

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Updated Go type checking behavior

When evaluating whether a given query variable matches a Go type Polar will now use direct instance comparisons instead of Go's `reflect.ConvertibleTo` functionality. This change resolves false-positive type checking results where discrete structs with identical sets of fields were considered to be equivalent.

This change has implications for the use of NewTypes in Polar rule definitions. Rules that are defined using NewTypes will now only match instances of the NewType and no longer match the underlying wrapped type.

Rules which consume NewTypes must now be specialized over the NewType directly and not the underlying wrapped type.

```go
type Action string
const (
    View Action = "view"
    Create Action = "create"
    Update Action = "update"
)
```

Where previously it was possible to utilize this `Role` type as interchangeable with that of `string`:
```polar
user_has_action(user: User, action: String, resource: Resource) if ...
```

This rule definition must be rewritten as follows:

```polar
user_has_action(user: User, action: Action, resource: Resource) if ...
```

### Python

#### Breaking Changes

##### `@polar_class` is deprecated in favor of `Oso#register_class`

The `@polar_class` decorator used to register classes with Polar has been deprecated. To register a class with Polar it is now necessary to use the [`Oso#register_class`](https://docs.osohq.com/reference/api/index.html#oso.Oso.register_class) API.

### `LANGUAGE`

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
