---
title: Quickstart
metaTitle: Quickstart for $LANG
description: |
  Ready to get started? See Oso in action, and walk through a quick
  change to an Oso policy in a simple web server.
any: false
weight: 1
---

{{< ifLang "rust" >}}
{{% callout "Rust quickstart coming soon" %}}

{{< coming_soon >}}

This guide uses Python.

{{% /callout %}}
<div class="pb-10"></div>
{{< /ifLang >}}


# Quickstart for {{% lang %}}

This guide will walk you through your first change to an Oso policy file. There
are three steps:

1. [Download](#1-clone-the-repo-and-install-dependencies) a minimal {{< lang >}}
   starter project that's already integrated with Oso.
2. [Run the server](#2-run-the-server) and visit the app in your browser.
3. [Make a small change](#3-update-the-policy) to the policy to allow a new type
   of access.

<!-- {{% minicallout %}} -->

The Oso Library works best in monolithic applications. If you're building authorization for more than one service or want to share a policy across multiple applications, read how to [get
started with Oso Cloud](https://www.osohq.com/docs/get-started/quickstart).

<!-- {{% /minicallout %}} -->

## 1. Clone the repo and install dependencies

First, clone [the {{< lang >}} quickstart repo]({{< exampleGet "githubUrl" >}}),
and install the dependencies:

```sh
git clone {{< exampleGet "githubCloneUrl" >}}
cd {{< exampleGet "repoName" >}}
{{< exampleGet "installDependencies" >}}
```

## 2. Run the server

With the dependencies installed, you should be ready to start the server:

```sh
{{< exampleGet "startServer" >}}
```

If all is well, the server should be listening on port {{< exampleGet "port"
>}}.

Visit [http://localhost:{{< exampleGet "port" >}}/repo/gmail](http://localhost:{{< exampleGet "port" >}}/repo/gmail)
in your browser. You should see a successful response, indicating that you have
access to the `gmail` repo.

To see an unsuccessful response, visit [http://localhost:{{< exampleGet "port" >}}/repo/react](http://localhost:{{<
exampleGet "port" >}}/repo/react). You'll see an error: `Repo named react was
not found`. There actually _is_ a repo named `react`, but you don't have access
to it. Let's fix that now.

## 3. Update the policy

In `{{< exampleGet "polarFileRelative" >}}`, add the following two lines to define a new "rule." This
rule will allow any "actor" (or user) to perform the `"read"` action on a
repository if that repository is marked as `{{< exampleGet "isPublic" >}}`.

<!-- NOTE: this doesn't use literalInclude only because we need to highlight the
addition of two lines.
This code should be kept in sync with examples/quickstart/**/*.polar. -->
{{< code file="main.polar" hl_lines="21-22" >}}
actor User {}

resource Repository {
  permissions = ["read", "push", "delete"];
  roles = ["contributor", "maintainer", "admin"];

  "read" if "contributor";
  "push" if "maintainer";
  "delete" if "admin";

  "maintainer" if "admin";
  "contributor" if "maintainer";
}

# This rule tells Oso how to fetch roles for a repository
{{< exampleGet "hasRole" >}}

has_permission(_actor: User, "read", repository: Repository) if
  repository.{{< exampleGet "isPublic" >}};

allow(actor, action, resource) if
  has_permission(actor, action, resource);

{{< /code >}}

Restart the server, and again visit [http://localhost:{{< exampleGet "port" >}}/repo/react](http://localhost:{{<
exampleGet "port" >}}/repo/react). Now, you'll see a successful response:

<img src="/getting-started/quickstart/react.png" style="max-width: 350px;
box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2); border-radius: 6px;" alt="A
200 response from /repo/react" />

## What just happened?

The quickstart server uses an Oso policy to make sure users are allowed to
view repos. The call to `{{< exampleGet "osoAuthorize" >}}` in `{{< exampleGet
"serverFileRelative" >}}` performs this check in {{% exampleGet "endpoint" %}}.
If the user does not have access to a repository, an error response is returned
to them.

In this case, the repo with the name `react` is public because of its definition
in the `{{< exampleGet "modelFileRelative" >}}` file, so it should be accessible
to everyone. By making the change to `{{< exampleGet "polarFileRelative" >}}`, you
told Oso to allow users to `"read"` repositories that have the `{{< exampleGet
"isPublic" >}}` field set to true.

That way, when you visited the `react` repo in your browser, Oso determined that
the action was permitted!

Check out the full code for the example below:

{{< literalInclude dynPath="serverFile" tabGroup="quickstart" >}}

{{< literalInclude dynPath="modelFile" tabGroup="quickstart" >}}

{{% callout "What's next" "blue" %}}

- [Add Oso to Your Application](application)

{{% /callout %}}
