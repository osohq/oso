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

Oso is a batteries-included framework for building authorization in your application.

With Oso, you can:
- **Model**: Set up common permissions patterns like role-based access control (RBAC) and relationships using Oso’s built-in primitives. Extend them however you need with Oso’s declarative policy language, Polar.
- **Filter**: Go beyond yes/no authorization questions. Implement authorization over collections too - e.g., “Show me only the records that Juno can see.”
- **Test**: Write unit tests over your authorization logic now that you have a single interface for it. Use the Oso debugger or REPL to track down unexpected behavior.

Oso offers libraries for [Node.js][badge-nodejs-link],
[Python][badge-python-link], [Go][badge-go-link],
[Rust][badge-rust-link], [Ruby][badge-ruby-link], and
[Java][badge-java-link].

Our latest creation Oso Cloud makes authorization across services as easy as oso.authorize(user, action, resource). [Learn about it.](https://www.osohq.com/oso-cloud)

## Documentation

- To get up and running with Oso, try the [Getting Started guide](https://docs.osohq.com/getting-started/quickstart.html).
- Full documentation is available at [docs.osohq.com](https://docs.osohq.com).
- Check out [Use Cases][use-cases] to learn more about how teams are using Oso in production.
- To learn about authorization best practices (not specific to Oso), read the [Authorization Academy](https://www.osohq.com/developers/authorization-academy) guides.

## Community & Support

If you have any questions on Oso or authorization more generally, you can join our engineering team & hundreds of other developers using Oso in our community Slack:

[![Button][join-slack-link]][badge-slack-link]

## Share your story

We'd love to hear about your use case and experience with Oso. Share your story in our [Success Stories issue](https://github.com/osohq/oso/issues/1081).

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

- Java: 11+
  - Maven: 3.6+
- Node.js: 12.20.0+
  - Yarn 1.22+
- Python: 3.7+
- Ruby: 2.4+
  - Bundler 2.1.4+
- Rust: 1.46+
- Go: 1.14+

## Contributing & Jobs

See: [CONTRIBUTING.md][contributing].

If you want to work on the Oso codebase full-time, visit [our jobs page](https://www.osohq.com/company/jobs).

## License

See: [LICENSE][license].

[join-slack-link]: https://user-images.githubusercontent.com/282595/128394344-1bd9e5b2-e83d-4666-b446-2e4f431ffcea.png
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
[use-cases]: https://www.osohq.com/use-cases
[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
