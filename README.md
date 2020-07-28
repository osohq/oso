# oso

[![Development][badge-ci]][badge-ci-link]
[![GitHub release (latest SemVer)][badge-release]][badge-release-link]
[![PyPI version][badge-python]][badge-python-link]
[![Gem Version][badge-ruby]][badge-ruby-link]
[![Slack][badge-slack]][badge-slack-link]

## What is oso?

oso is an **open source policy engine for authorization** that’s embedded in
your application. It provides a declarative policy language for expressing
authorization logic, which you define separately from the rest of your
application code but which executes inside the application and can call
directly into it. oso ships as a library with a built-in debugger and REPL.

## Getting started

To get up and running with oso, check out the [Getting Started guide][docs] in
the [oso documentation][docs].

## Development

### Core

oso's Rust core is developed against [Rust's latest stable release][rust].

### Language libraries

oso's language libraries can be developed without touching the Rust core, but
you will still need the Rust stable toolchain installed in order to build the
core.

#### Language requirements

To work on a language library, you will need to meet the following version
requirements:

- Java: 10+
- Python: 3.6+
- Ruby: 2.4+

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
<!-- NOTE: the Slack invite link must be recreated every 30 days or every 2000
invites, whichever comes first. -->
[badge-slack-link]: https://join.slack.com/t/oso-oss/shared_invite/zt-g8asdmdt-26n_E1TjBxa64J17oXv~~A
[contributing]: https://github.com/osohq/oso/blob/main/CONTRIBUTING.md
[docs]: https://docs.osohq.com
[license]: https://github.com/osohq/oso/blob/main/LICENSE
[rust]: https://www.rust-lang.org/tools/install
[core]: https://github.com/osohq/oso/tree/main/polar
[languages-java]: https://github.com/osohq/oso/tree/main/languages/java
[languages-python]: https://github.com/osohq/oso/tree/main/languages/python
[languages-ruby]: https://github.com/osohq/oso/tree/main/languages/ruby
