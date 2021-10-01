---
title: Installation
weight: 1
description: Installation instructions for Oso's Go library.
aliases: 
  - ./lib.html
---

# Installation

The Go version of Oso is available on
[go.dev](https://pkg.go.dev/github.com/osohq/go-oso).

It can be added as a dependency to a Go project:

```console
go get github.com/osohq/go-oso@v{{< version >}}
```

And imported into a Go file:

```go
import "github.com/osohq/go-oso"
```

## Requirements

- Go version 1.12 or higher
- Supported platforms (x64 only):
  - Linux
  - macOS
    - Go 1.12.7 is the minimum supported version on macOS due to [this cgo
      issue][cgo issue].
  - Windows

{{% minicallout %}}
  **Note**: Oso depends on [cgo][], and on Windows cgo depends on a [MinGW
  toolchain][tdm-gcc].
{{% /minicallout %}}

[cgo issue]: https://github.com/golang/go/issues/32697
[cgo]: https://pkg.go.dev/cmd/cgo
[tdm-gcc]: https://jmeubank.github.io/tdm-gcc/
