---
weight: 2
title: SQLAlchemy Library
aliases:
    - /using/frameworks/sqlalchemy.html
description: The Oso SQLAlchemy integration enables data filtering and provides built-in models for RBAC.
---

# SQLAlchemy

The Oso SQLAlchemy integration enables data filtering
to be used with SQLAlchemy models and provides built-in SQLAlchemy models for
Role-Based Access Control.

{{% callout "Adding roles to your application with SQLAlchemy?" "blue" %}}

We just released early access to the next version of our roles
library.

[Check it out here!](/new-roles)

{{% /callout %}}

## Installation

The Oso SQLAlchemy integration is available on [PyPI](https://pypi.org/project/sqlalchemy-oso/) and can be installed using
`pip`:

```
$ pip install sqlalchemy-oso
```

## Usage

See the [data filtering guide](guides/data_access/sqlalchemy) and [roles guide](guides/roles/sqlalchemy_roles) for usage information.

## API Reference

The [SQLAlchemy API reference]({{% apiLink "reference/api/sqlalchemy.html" %}})
is automatically generated from the Oso SQLAlchemy library source files.

## Roles Reference

The SQLAlchemy Roles Reference has more details on the library's built-in roles features.
