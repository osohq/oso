---
title: "Get Started"
weight: 1
any: true
---

# Get Started

Let's get started with Oso! Oso is a framework for application
authorization with built-in primitives for modeling authorization. Oso
consists of the following components:

- **Oso Library**: Oso is a library that you include in your application to
enforce authorization. The library supports multiple languages,
currently Python, Node.js, Ruby, Go, Java, and Rust.
- **Polar Policy Language**: With Oso, you express authorization logic
declaratively using Polar, our policy language. The Oso library
evaluates policies to make authorization decisions. Polar policies are
written directly over the same data types that you use in your
application.
    - **Resources, Roles and Permissions**: The most common way to model
    authorization with Oso is through resources. Specify resources in
    Polar and the roles and permissions you want users to have on them.
    - **Rules**: Since Polar is a declarative language, you can extend
    the resource model to cover other cases as needed by your
    application. For example, you may deny access by banned users or
    allow any user to access a public resource.
{{% ifLang "python" %}}
- **Framework Libraries**: Oso has pre-built integrations for web
application libraries, including Django, SQLAlchemy, and Flask.
{{% /ifLang %}}

To quickly have a runnable example of using Oso, check out our
[Quickstart](quickstart) where you can see a sample app with Oso.

To use Oso for authorization in an existing application, check out our
[Add Oso to Your App guide](application), where you'll be introduced to
enforcing authorization, setting up role based authorization, writing
rules, and filtering data by authorization.

For more on the design principles behind Oso and why it makes
authorization easy for development teams see the **TODO** What is Oso
section or **TODO** read stories from teams using Oso — like Wayfair and
Intercom.
