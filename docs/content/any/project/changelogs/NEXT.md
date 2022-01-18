---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (oso 0.26.0, VS Code extension 0.26.0)
  containing new features, bug fixes, and more.
draft: true
---

## `oso` 0.26.0

### Core

#### Other bugs & improvements

- Fixed a bug affecting runtime type checking on nested object attributes.
  
### Python

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### `@polar_class` is deprecated in favor of `Oso#register_class`

The `@polar_class` decorator used to register classes with Polar has been deprecated. To register a class with Polar it is now necessary to use the [`Oso#register_class`](https://docs.osohq.com/reference/api/index.html#oso.Oso.register_class) API.

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().

## VS Code extension 0.26.0

### New features

#### Configuring which Polar files are treated as part of the same policy

The `oso.polarLanguageServer.projectRoots` VS Code workspace configuration
setting can be used to control which Polar files in a particular workspace
folder are treated as part of the same Oso policy. For more details, see [the
docs](reference/tooling/ide#configuring-which-polar-files-are-treated-as-part-of-the-same-policy).
