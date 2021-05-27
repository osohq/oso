---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `sqlalchemy-oso-preview` 0.0.5

### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

#### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

### New features

#### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

### Other bugs & improvements

- Calling `sqlalchemy_oso.SQLAlchemyOso.enable_roles()` more than once will now
  raise an error. There's no need to call the method multiple times.

- Using a global SQLAlchemy declarative base class would previously
  result in some issues when reusing the same base class across multiple
  `sqlalchemy_oso.SQLAlchemyOso` instances, e.g., when running multiple tests
  that construct new `SQLAlchemyOso` instances but reuse the same global
  declarative base class. The issues are now fixed by ignoring internal models
  when verifying that all models descending from the given base class have
  primary keys of the same type. For more on that requirement, see [the version
  0.0.4 changelog](project/changelogs/2021-05-26).
