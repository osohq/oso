# oso

[![Development][badge-ci]][badge-ci-link]
[![GitHub release (latest SemVer)][badge-release]][badge-release-link]
[![PyPI version][badge-python]][badge-python-link]
[![Gem Version][badge-ruby]][badge-ruby-link]
[![Slack][badge-slack]][badge-slack-link]

## What is oso?

oso is an **open source policy engine for authorization** that’s embedded in
your application. It provides a declarative policy language for expressing
authorization logic. You define this logic separately from the rest of your
application code, but it executes inside the application and can call
directly into it. oso ships as a library with a built-in debugger and REPL.

oso is ideal for building permissions into user-facing applications, but you can
check out [Use Cases](https://docs.osohq.com/more/use-cases.html) to learn about
other applications for oso.

Using oso consists of two parts:

1. Writing oso policies in a declarative policy language called Polar
2. Embedding oso in your application using the appropriate language-specific authorization library

## Getting started

To get up and running with oso, check out the [Getting Started guides](https://docs.osohq.com/getting-started/quickstart.html) in the [oso documentation][docs].

## Development

### Core

oso's Rust core is developed against [Rust's latest stable release][rust].

### Language libraries

oso's language libraries can be developed without touching the Rust core, but
you will still need the Rust stable toolchain installed in order to build the
core.

To build the WebAssembly core for the Node.js library, you will need to have
[`wasm-pack`][wasm-pack] installed and available on your system PATH.

#### Language requirements

To work on a language library, you will need to meet the following version
requirements:

- Java: 10+
  - Maven: 3.6+
- Python: 3.6+
- Ruby: 2.4+
  - Bundler 2.1.4+
- Node.js: 10.14.2+
  - Yarn 1.22+

## Contributing

See: [CONTRIBUTING.md][contributing].

## License

See: [LICENSE][license].

[badge-ci]: https://github.com/osohq/oso/workflows/Development/badge.svg
[badge-ci-link]: https://github.com/osohq/oso/actions?query=branch%3Amain+workflow%3ADevelopment
[badge-release]: https://img.shields.io/github/v/release/osohq/oso?color=005b96&logo=github&sort=semver
[badge-release-link]: https://github.com/osohq/oso/releases
[badge-ruby]: https://badge.fury.io/rb/oso-oso.svg
[badge-ruby-link]: https://rubygems.org/gems/oso-oso
[badge-python]: https://badge.fury.io/py/oso.svg
[badge-python-link]: https://pypi.org/project/oso/
[badge-slack]: https://img.shields.io/badge/slack-oso--oss-orange
[badge-slack-link]: https://join-slack.osohq.com/
[contributing]: https://github.com/osohq/oso/blob/main/CONTRIBUTING.md
[docs]: https://docs.osohq.com
[license]: https://github.com/osohq/oso/blob/main/LICENSE
[rust]: https://www.rust-lang.org/tools/install
[core]: https://github.com/osohq/oso/tree/main/polar
[languages-java]: https://github.com/osohq/oso/tree/main/languages/java
[languages-python]: https://github.com/osohq/oso/tree/main/languages/python
[languages-ruby]: https://github.com/osohq/oso/tree/main/languages/ruby
[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
