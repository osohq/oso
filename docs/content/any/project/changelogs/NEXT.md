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

## `sqlalchemy-oso` 0.26.3

### Other bugs & improvements

- A missing version constraint on the Flask-SQLAlchemy extra allowed
  Flask-SQLAlchemy versions greater than 2.x to be used with `sqlalchemy-oso`.
  The `sqlalchemy-oso` library requires some updates for compatibility with
  Flask-SQLAlchemy 3.0, and progress on those updates will be tracked in
  https://github.com/osohq/oso/issues/1631. Until compatibility with
  Flask-SQLAlchemy 3.0 is achieved, we've added a runtime check on the
  Flask-SQLAlchemy version that will raise an error if an incompatible version
  is found. Thanks to [`@snstanton`](https://github.com/snstanton) for the
  report and PR!
