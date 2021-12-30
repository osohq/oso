---
title: IDE Support
aliases:
    - ../../more/dev-tools/ide.html
description: IDE integrations for working with Oso policies.
---

# IDE Support

Oso’s IDE (Integrated Development Environment) integrations provide syntax
highlighting for `.polar` files. Additionally, our Visual Studio Code extension
will display diagnostics (errors & warnings) from your Oso policy in-line in
the editor and in VS Code's **Problems** pane.

## Supported IDEs

### [Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=osohq.oso)

#### Features

- Syntax highlighting.
- Diagnostics (errors & warnings) from your Oso policy are displayed in-line in
  the editor and in VS Code's **Problems** pane.
  - The extension immediately highlights errors and warnings encountered while
    parsing and validating your policy, such as if a rule is missing a trailing
    semi-colon, a resource block declares `"owner"` as both a role and a
    relation, or your policy contains no `allow()` rule. You would normally see
    this feedback when running your application, but the extension surfaces it
    while you edit your policy.

### [Vim](https://github.com/osohq/polar.vim)

#### Features

- Syntax highlighting.

### Want support for your IDE of choice?

Let us know by using our chat on the bottom right to send us a message.
