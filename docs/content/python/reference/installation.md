---
title: Installation
weight: 1
description: Installation instructions for Oso's Python library.
aliases:
  - /using/libraries/python/api.html
  - ./lib.html
---

# Installation

The Python version of Oso is available on [PyPI](https://pypi.org/project/oso/)
and can be installed using `pip`:

```console
$ pip install oso=={{< version >}}
```

## Requirements

- Python version 3.6 or greater
- Supported platforms:
  - Linux
  - macOS
  - Windows

{{% minicallout %}}
  **Note**: The standard Python package is known to work on glibc-based
  distributions but not on musl-based ones like Alpine Linux. Wheels built
  against musl that you can use on Alpine Linux can be downloaded from [the
  releases page on GitHub][releases].
{{% /minicallout %}}

[releases]: https://github.com/osohq/oso/releases/latest

## Framework & ORM Integrations

Oso also provides [libraries](frameworks) to integrate with popular Python
frameworks and ORMS.
