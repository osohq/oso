---
title: Python Library
weight: 2
any: false
aliases:
    - /using/libraries/python/api.html
---

# Python Authorization Library

The Python version of Oso is available on [PyPI](https://pypi.org/project/oso/)
and can be installed using `pip`:

```console
$ pip install oso
```

## Requirements

* Python version 3.6 or greater
* Supported platforms:
  * Linux
  * macOS
  * Windows

The standard Python package is known to work on glibc-based distributions, but
not on musl-based ones like Alpine Linux. Wheels built against musl that you
can use on Alpine Linux can be downloaded from [the releases page on
GitHub](https://github.com/osohq/oso/releases/latest).

## Python API

The [Python API reference]({{% apiLink "reference/api/index.html" %}}) is
automatically generated from the Oso Python library source files.

## Framework & ORM Integrations

Oso also provides [libraries](frameworks) to integrate with popular Python
frameworks and ORMS.

{{% callout "Adding roles to your application with SQLAlchemy?" "blue" %}}

We just released early access to the next version of our roles
library.

[Check it out here!](/guides/new-roles)

{{% /callout %}}
