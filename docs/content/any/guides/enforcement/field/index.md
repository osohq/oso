---
title: "Field-level Enforcement"
weight: 30
any: true
description: >
  Learn about enforcing field-level authorization, controlling who can access
  which fields of your resources.
---

# Field-level Enforcement

Perhaps you're building a `/profile` endpoint, and you'd like to exclude the
profile's email address unless the current user specifically has access to it.
To build this type of authorization, it often makes sense to use **"field-level"
enforcement**, by explicitly allowing access to certain fields on your domain
objects.

Field-level authorization gives you fine-grained control over who can access
exactly what bits of information. In Polar, you can write field-level rules like
this:

```polar
allow_field(user, "read", profile: Profile, "email") if
    user = profile.user or
    user.{{< exampleGet "isAdmin" >}};
```

Notice that an `allow_field` rule is just like an `allow` rule, except that it
takes an additional argument: the field name.

## Authorize one field at a time

To enforce field-level authorization in your app, you use the {{% apiDeepLink
class="Oso" %}}{{< exampleGet "authorizeField" >}}{{% /apiDeepLink %}} method.

{{% exampleGet "getEmail" %}}

Like `{{< exampleGet "authorize" >}}`, `{{< exampleGet "authorizeField" >}}`
will raise an an authorization error when the
user is not allowed to perform the given action. This is an error that you
should handle globally in your app. You can read more details about this in the
[Resource-level Enforcement Guide](resource.html#authorization-failure).

## Get all authorized fields

Sometimes it is helpful to get _all_ fields that a user can access, and for this
there is a separate method called {{% apiDeepLink class="Oso"
%}}{{< exampleGet "authorizedFields" >}}{{%/apiDeepLink %}}:

{{% exampleGet "serializeProfile" %}}

The `{{< exampleGet "authorizedFields" >}}` method can be used to send only the fields that the user
is explicitly allowed to read, or can similarly be used to filter _incoming_
parameters from a user for a call to, say, an `update` method. In that case, you
might use an `"update"` action in the call to `{{< exampleGet "authorizedFields" >}}`:

{{% exampleGet "filterUpdateParams" %}}

## Authorizing many fields

Perhaps you have many fields on each object, and you'd like to allow access to
them in groups. For example, a `Profile` object might have some public fields,
some fields viewable only by friends, and some fields viewable by admins only.

You can do this with Polar's `in` operator:

```polar
# Allow friends access to friend-only fields
allow_field(user: User, "read", profile: Profile, field) if
    field in {{< exampleGet "fieldsFriendsOnlyBefore" >}} and
    user in profile.friends;

# Allow admins access to admin-only fields
allow_field(user: User, "read", profile: Profile, field) if
    field in {{< exampleGet "fieldsAdminOnlyBefore" >}} and
    user.{{< exampleGet "isAdmin" >}};
```

Or, if you have trouble listing all fields in your Polar policy files, and you'd
prefer to list fields in your application code, you can also use a constant
defined on the class, like this:

```polar
allow_field(user: User, "read", profile: Profile, field) if
    field in {{< exampleGet "fieldsFriendsOnlyAfter" >}} and
    user in profile.friends;

allow_field(user: User, "read", profile: Profile, field) if
    field in {{< exampleGet "fieldsAdminOnlyAfter" >}} and
    user.{{< exampleGet "isAdmin" >}};
```

{{% exampleGet "fieldDefinitions" %}}

That way, you can add new fields and authorize access to them without touching
your Polar policy code.
