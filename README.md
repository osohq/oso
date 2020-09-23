# oso

[![Development][badge-ci]][badge-ci-link]
[![GitHub release (latest SemVer)][badge-release]][badge-release-link]
[![Maven version][badge-java]][badge-java-link]
[![NPM version][badge-nodejs]][badge-nodejs-link]
[![PyPI version][badge-python]][badge-python-link]
[![RubyGems version][badge-ruby]][badge-ruby-link]
[![Crates.io version][badge-rust]][badge-rust-link]
[![Slack][badge-slack]][badge-slack-link]

## What is oso?

oso is an **open source policy engine for authorization** that’s embedded in
your application. It provides a declarative policy language for expressing
authorization logic. You define this logic separately from the rest of your
application code, but it executes inside the application and can call directly
into it. oso ships as a library with a built-in debugger and REPL.

oso is ideal for building permissions into user-facing applications, but you
can check out [Use Cases][use-cases] to learn about other applications for oso.

Using oso consists of two parts:

1. Writing oso policies in a declarative policy language called Polar.
2. Embedding oso in your application using the appropriate language-specific
   authorization library.

oso currently offers libraries for [Java][badge-java-link],
[Node.js][badge-nodejs-link], [Python][badge-python-link],
[Ruby][badge-ruby-link], and [Rust][badge-rust-link].

## Getting started

To get up and running with oso, check out the [Getting Started
guides](https://docs.osohq.com/getting-started/quickstart.html) in the [oso
documentation][docs].

## Development

### Core

oso's Rust [core][core] is developed against [Rust's latest stable
release][rust].

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
- Node.js: 10.14.2+
  - Yarn 1.22+
- Python: 3.6+
- Ruby: 2.4+
  - Bundler 2.1.4+
- Rust: 1.46+

## Contributing

See: [CONTRIBUTING.md][contributing].

## License

See: [LICENSE][license].

[badge-ci]: https://github.com/osohq/oso/workflows/Development/badge.svg
[badge-ci-link]: https://github.com/osohq/oso/actions?query=branch%3Amain+workflow%3ADevelopment
[badge-release]: https://img.shields.io/github/v/release/osohq/oso?color=005b96&logo=github&sort=semver
[badge-release-link]: https://github.com/osohq/oso/releases
[badge-slack]: https://img.shields.io/badge/slack-oso--oss-orange
[badge-slack-link]: https://join-slack.osohq.com/

[badge-java]: https://img.shields.io/maven-central/v/com.osohq/oso
[badge-java-link]: https://search.maven.org/artifact/com.osohq/oso
[badge-nodejs]: https://badge.fury.io/js/oso.svg
[badge-nodejs-link]: https://www.npmjs.com/package/oso
[badge-python]: https://badge.fury.io/py/oso.svg
[badge-python-link]: https://pypi.org/project/oso/
[badge-ruby]: https://badge.fury.io/rb/oso-oso.svg
[badge-ruby-link]: https://rubygems.org/gems/oso-oso
[badge-rust]: https://img.shields.io/crates/v/oso
[badge-rust-link]: https://crates.io/crates/oso

<!-- [languages-java]: https://github.com/osohq/oso/tree/main/languages/java -->
<!-- [languages-nodejs]: https://github.com/osohq/oso/tree/main/languages/js -->
<!-- [languages-python]: https://github.com/osohq/oso/tree/main/languages/python -->
<!-- [languages-ruby]: https://github.com/osohq/oso/tree/main/languages/ruby -->
<!-- [languages-rust]: https://github.com/osohq/oso/tree/main/languages/rust -->

[contributing]: https://github.com/osohq/oso/blob/main/CONTRIBUTING.md
[core]: https://github.com/osohq/oso/tree/main/polar-core
[docs]: https://docs.osohq.com
[license]: https://github.com/osohq/oso/blob/main/LICENSE
[rust]: https://www.rust-lang.org/tools/install
[use-cases]: https://docs.osohq.com/more/use-cases.html
[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
