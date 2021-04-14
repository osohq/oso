---
title: Go Library
weight: 2
any: false
---

# Go Authorization Library

The Go version of Oso is available on
[go.dev](https://pkg.go.dev/github.com/osohq/go-oso).

It can be added as a dependency to a Go project:

```console
go get github.com/osohq/go-oso
```

And imported into a Go file:

```go
import "github.com/osohq/go-oso"
```

For more information on the Oso Go library, see the library documentation.

## Requirements

* Go version 1.12 or higher
* Supported platforms (x64 only):
  * Linux
  * macOS
  * Windows

Oso uses cgo to embed our VM and on Windows cgo depends on a [MinGW
toolchain](https://jmeubank.github.io/tdm-gcc/).
