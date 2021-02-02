---
title: Application types
weight: 2
any: false
aliases: 
    - /getting-started/policies/application-types.html
---

# Application Types

Any type defined in an application can be passed into oso, and its attributes
may be accessed from within a policy. Using application types make it possible
to take advantage of an app’s existing domain model. For example:

```polar
allow(actor, action, resource) if actor.{{% exampleGet "isAdmin" %}};
```

<!-- TODO(gj): Link `Oso.isAllowed()` once API docs are setup. -->
The above rule expects the `actor` variable to be a {{% exampleGet "langName"
%}} {{% exampleGet "instance" %}} with the field `{{% exampleGet "isAdmin"
%}}`. The {{% exampleGet "langName" %}} {{% exampleGet "instance" %}} is passed
into oso with a call to `Oso.{{% exampleGet "isAllowed" %}}()`:

{{% exampleGet "userClass" %}}

The code above provides a `User` object as the *actor* for our `allow` rule.
Since `User` has a field called `{{% exampleGet "isAdmin" %}}`, it is checked
during evaluation of the Polar rule and found to be true.

In addition to accessing attributes, you can also call methods on application
instances in a policy:

```polar
allow(actor, action, resource) if actor.{{% exampleGet "isAdminOf" %}}(resource);
```

## Registering Application Types

Instances of application types can be constructed from inside an oso policy
using [the `new` operator](polar-syntax#new) if the class has been
**registered**. {{% exampleGet "registerClass" %}}

Once the class is registered, we can make a `User` object in Polar. This can be
helpful for writing inline test queries:

{{% exampleGet "testQueries" %}}

Registering classes also makes it possible to use
[specialization](polar-syntax#specialization) and [the `matches`
operator](polar-syntax#matches-operator) with the registered class.

In our previous example, the **allow** rule expected the actor to be a `User`,
but we couldn’t actually check that type assumption in the policy. If we
register the `User` class, we can write the following rule:

```polar
allow(actor: User, action, resource) if actor.name = "alice";
```

This rule will only be evaluated when the actor is a `User`; the `actor`
argument is *specialized* on that type. We could also use `matches` to express
the same logic on an unspecialized rule:

```polar
allow(actor, action, resource) if actor matches User{name: "alice"};
```

Either way, using the rule could look like this:

{{% exampleGet "specializedExample" %}}

{{< callout "Note" "green" >}}
  Type specializers automatically respect the **inheritance** hierarchy of
  application classes. See the [Resources with
  Inheritance](learn/policies/examples/inheritance) guide for an in-depth
  example of how this works.
{{< /callout >}}

Once a class is registered, class or static methods can also be called from oso
policies:

```polar
allow(actor: User, action, resource) if actor.name in User.superusers();
```

{{% exampleGet "classMethodExample" %}}

## Built-in Types

Methods called on the Polar built-in types `String`, `Dictionary`, `Number`,
and `List` punt to methods on the corresponding application language class.
That way you can use familiar methods like `.{{% exampleGet "startswith" %}}()`
on strings regardless of whether they originated in your application or as a
literal in your policy. This applies to all of Polar's [supported
types](polar-syntax#primitive-types) in any supported application language. For
examples using built-in types, see [the {{% exampleGet "langName" %}}
library](reference/classes) guide.

{{< callout "Warning" "orange" >}}
  Do not attempt to mutate a literal using a method on it. Literals in Polar
  are constant, and any changes made to such objects by calling a method will
  not be persisted.
{{< /callout >}}

### `nil`

In addition to the built-in types, Polar registers a constant named
`nil` that maps to an application-language-specific “null” value:

| Polar | Python | Ruby  | Java   | JavaScript | Rust                | SQL    |
| ----- | ------ | ----- | ------ | ---------- | ------------------- | ------ |
| `nil` | `None` | `nil` | `null` | `null`     | `Option::<T>::None` | `NULL` |

The Polar value `nil` is not equal to either the empty list `[]`
or the boolean value `false`. It is intended to be used with application
methods that return a null value.

## Summary

* **Application types** and their associated application data are available
  within policies.
* Types can be **registered** with oso, in order to:
  * Create instances of application types in policies
  * Leverage the inheritance structure of application types with **specialized
    rules**, supporting more sophisticated access control models.
* You can use **built-in methods** on primitive types & literals like strings
  and dictionaries, exactly as if they were application types.
