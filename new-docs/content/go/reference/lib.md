---
title: Go Library
weight: 4
any: false
---
# Go Authorization Library

The go version of Oso is available on [go.dev](https://pkg.go.dev/github.com/osohq/go-oso).

It can be added as a dependency to a go project:

```
go get github.com/osohq/go-oso
```

And imported into a go file.

```
import "github.com/osohq/go-oso"
```

For more information on the Oso go library, see the
library documentation.

**Requirements**

* Go version 1.12 or higher
* Supported platforms (x64 only):
  * Linux
  * OS X
  * Windows

Oso uses cgo to embed our vm and on windows cgo depends on a [[MinGW toolchain]](https://jmeubank.github.io/tdm-gcc/).
