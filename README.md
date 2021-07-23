# Oso

[![Development][badge-ci]][badge-ci-link]
[![GitHub release (latest SemVer)][badge-release]][badge-release-link]
[![Go version][badge-go]][badge-go-link]
[![Maven version][badge-java]][badge-java-link]
[![NPM version][badge-nodejs]][badge-nodejs-link]
[![PyPI version][badge-python]][badge-python-link]
[![RubyGems version][badge-ruby]][badge-ruby-link]
[![Crates.io version][badge-rust]][badge-rust-link]
[![Slack][badge-slack]][badge-slack-link]

## What is Oso?

Oso is a batteries-included library for building authorization in your application.

Oso gives you a mental model and an authorization system – a set of APIs built on top of a declarative policy language called Polar, plus a debugger and REPL – to define who can do what in your application. You can express common concepts from “users can see their own data” and role-based access control, to others like multi-tenancy, organizations and teams, hierarchies and relationships.

Oso lets you offload the thinking of how to design authorization and build features fast, while keeping the flexibility to extend and customize as you see fit.

Developers can typically write a working Oso policy in <5 minutes, add Oso to an app in <30 minutes, and use Oso to solve real authorization problems within a few hours. To get started, you add the library to your application, create a new Oso instance and load an Oso policy. You can mix and match any of Oso’s authorization APIs to implement features like roles with custom policies that you write to suit your application.

Oso is ideal for building permissions into user-facing applications, but you
can check out [Use Cases][use-cases] to learn about other applications for Oso.

Oso currently offers libraries for [Node.js][badge-nodejs-link], [Python][badge-python-link], [Go][badge-go-link], 
[Rust][badge-rust-link], [Ruby][badge-ruby-link], and [Java][badge-java-link].

## Getting started

To get up and running with Oso, check out the [Getting Started
guides](https://docs.osohq.com/getting-started/quickstart.html) in the [Oso
documentation][docs].

If you have questions, need help getting started, or want to discuss anything about the product, your use case, or authorization more generally, [join us on Slack][badge-slack-link].

## Development

### Core

Oso's Rust [core][core] is developed against [Rust's latest stable
release][rust].

### Language libraries

Oso's language libraries can be developed without touching the Rust core, but
you will still need the Rust stable toolchain installed in order to build the
core.

To build the WebAssembly core for the Node.js library, you will need to have
[`wasm-pack`][wasm-pack] installed and available on your system PATH.

#### Language requirements

To work on a language library, you will need to meet the following version
requirements:

- Java: 10+
  - Maven: 3.6+
- Node.js: 12.20.0+
  - Yarn 1.22+
- Python: 3.6+
- Ruby: 2.4+
  - Bundler 2.1.4+
- Rust: 1.46+
- Go: 1.12+

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
[badge-go]: https://img.shields.io/github/v/tag/osohq/go-oso?color=7fd5ea&label=go.dev
[badge-go-link]: https://pkg.go.dev/github.com/osohq/go-oso
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
[go-link]: https://pkg.go.dev/github.com/osohq/go-oso
[contributing]: https://github.com/osohq/oso/blob/main/CONTRIBUTING.md
[core]: https://github.com/osohq/oso/tree/main/polar-core
[docs]: https://docs.osohq.com
[license]: https://github.com/osohq/oso/blob/main/LICENSE
[rust]: https://www.rust-lang.org/tools/install
[use-cases]: https://docs.osohq.com/more/use-cases.html
[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/

## Share your story

We'd love to hear about your use case and experience with Oso. Share your story on [Twitter](https://twitter.com/osoHQ) or fill out [this form](https://osohq.typeform.com/to/mIFfkN05) for some Oso swag.
