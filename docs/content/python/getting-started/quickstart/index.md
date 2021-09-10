---
title: Quickstart (5 min)
description: |
  Ready to get started? See Oso in action, and walk through our quick
  tutorial for adding authorization to a simple web server.
weight: 1
---

<!--

This guide is not setup to use literalInclude. As a result the
examples are manually maintained to match the quickstart repository.

This needs to be updated.

-->

# Quickstart

This guide will walk you through your first change to an Oso policy file. There
are three steps:

1. [Download](#1-clone-the-repo-and-install-dependencies) a minimal {{< lang >}}
   starter project that's already integrated with Oso.
2. [Run the server](#2-run-the-server) and verify that it works.
3. [Make a small change](#3-update-the-policy) to the policy to allow a new type
   of access.

## 1. Clone the repo and install dependencies


```sh
git clone {{< exampleGet "githubUrl" >}}
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
not found`. There _is_ actually a repo named `react`, but you don't have access
to it. Let's fix that now.

## 3. Update the policy

In `app/main.polar`, add the following two lines to define a new "rule." This
rule will allow any "actor" (or user) to perform the `"read"` action on a
repository if that repository is marked as "public".

{{< code file="main.polar" highlight="21-22" syntax=diff >}}
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

 # This rule tells Oso how to fetch roles for a user
 has_role(actor, role_name, repository: Repository) if
   role in actor.roles and
   role_name = role.name and
   repository = role.repository;

+allow(_actor, "read", repository: Repository) if
+  repository.public;

 allow(actor, action, resource) if
   has_permission(actor, action, resource);

{{< /code >}}

Restart the server, and again visit [http://localhost:{{< exampleGet "port" >}}/repo/react](http://localhost:{{<
exampleGet "port" >}}/repo/react). Now, you'll see a successful response:

<img src="/getting-started/quickstart/react.png" style="max-width: 350px;
box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2); border-radius: 6px;" alt="A
200 response from /repo/react" />

## What just happened?




{{< literalInclude dynPath="serverFile" tabGroup="quickstart" >}}

{{< literalInclude dynPath="modelFile" tabGroup="quickstart" >}}

## Want to talk it through?

If you have any questions, are getting stuck, or just want to talk something
through, jump into [Slack](https://join-slack.osohq.com/) and an engineer from
the core team (or one of the hundreds of developers in the growing community)
will help you out.

```console
git clone {{% exampleGet "githubURL" %}}
```

{{% callout "What's next" "blue" %}}

- Explore how to [add Oso to an application](application).
- Dive into [writing policies](policies) in detail.

{{% /callout %}}
