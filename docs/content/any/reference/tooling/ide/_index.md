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

#### Configuration

##### Configuring which Polar files are treated as part of the same policy

By default, the extension assumes that each VS Code workspace folder contains a
separate Oso policy. On startup, the extension searches each workspace folder
for `.polar` files and treats all files it finds in a particular workspace
folder as part of the same policy.

This can sometimes lead to the extension having an incorrect view of a policy.
For example, if a workspace folder contains `src/` and `dist/` directories that
contain duplicate copies of the same Polar files, by default the extension will
treat the duplicates as part of the same policy. Or if a workspace folder
contains two separate projects with separate policies, `microservice-a/` and
`microservice-b/`, we don't want the extension to treat all of those Polar
files as parts of a whole.

The `oso.polarLanguageServer.projectRoots` VS Code configuration property can
be used to customize the extension's view of the various Oso policies in a
particular workspace folder. It accepts a list of **relative, POSIX-style**
paths that indicate the Oso 'project roots' present in a particular workspace
folder.

{{% minicallout %}}
  Because the configuration pertains to a particular workspace, it makes the
  most sense to configure it in *Workspace Settings* and not *User Settings*.
{{% /minicallout %}}

For the first example above, the following configuration includes Polar files
in the `src/` directory and ignores those in the `dist/` directory:

```json
{
  "oso.polarLanguageServer.projectRoots": [
    "./src"
  ]
}
```

For the second example above, the following configuration specifies that
`microservice-a` and `microservice-b` contain two separate Oso policies:

```json
{
  "oso.polarLanguageServer.projectRoots": [
    "./microservice-a",
    "./microservice-b"
  ]
}
```

### [Vim](https://github.com/osohq/polar.vim)

#### Features

- Syntax highlighting.

### Want support for your IDE of choice?

Let us know by using our chat on the bottom right to send us a message.
