---
title: "Field-level Enforcement"
weight: 30
any: true
description: >
  Learn about enforcing field-level authorization, controlling who can access
  which fields of your resources.
# draft: True
---

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

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
    user.is_admin;
```

Notice that an `allow_field` rule is just like an `allow` rule, except that it
takes an additional argument: the field name.

## The `authorize_field` method

To enforce field-level authorization in your app, you use the {{% apiDeepLink
class="Oso" %}}authorize_field{{% /apiDeepLink %}} method.

```python
def get_last_check_in_location(profile, current_user):
    oso.authorize_field(current_user, "read", profile, "email")
    return profile.email
```

Like `authorize`, `authorize_field` will raise an `AuthorizationError` when the
user is not allowed to perform the given action. This is an error that you
should handle globally in your app. You can read more details about this in the
[Resource-level Enforcement Guide](resource.html#authorization-failure).

## Get all authorized fields

Sometimes it is helpful to get _all_ fields that a user can access, and for this
there is a separate method called {{% apiDeepLink class="Oso"
%}}authorized_fields{{%/apiDeepLink %}}:

```python
# Serialize only the fields of profile that the current user is allowed to read
def serialize_profile(profile, current_user):
    fields = oso.authorized_fields(current_user, "read", profile)
    return {field: profile[field] for field in fields}
```

The `authorized_fields` method can be used to send only the fields that the user
is explicitly allowed to read, or can similarly be used to filter _incoming_
parameters from a user for a call to, say, an `update` method. In that case, you
might use an `"update"` action in the call to `authorized_fields`:

```python
# Filter update_params by the fields on profile that the user can update
def filter_update_params(profile, raw_update_params, current_user):
    fields = oso.authorized_fields(current_user, "update", profile)
    return {field: raw_update_params[field] for field in fields}
```

## Authorizing many fields

Perhaps you have many fields on each object, and you'd like to allow access to
them in groups. For example, a `Profile` object might have some public fields,
some fields viewable only by friends, and some fields viewable by admins only.

You can do this with Polar's `in` operator:

```python
# Allow friends access to friend-only fields
allow_field(user: User, "read", profile: Profile, field) if
    field in ["last_check_in_location", "favorite_animal"] and
    user in profile.friends;

# Allow admins access to admin-only fields
allow_field(user: User, "read", profile: Profile, field) if
    field in ["email", "last_login"] and
    user.is_admin;
```

Or, if you have trouble listing all fields in your Polar policy files, and you'd
prefer to list fields in your application code, you can also use a constant
defined on the class, like this:

```python
allow_field(user: User, "read", profile: Profile, field) if
    field in Profile.FRIENDS_ONLY_FIELDS and
    user in profile.friends;

allow_field(user: User, "read", profile: Profile, field) if
    field in Profile.ADMIN_ONLY_FIELDS and
    user.is_admin;
```

Doing so would require you to add the `FRIENDS_ONLY_FIELDS` and
`ADMIN_ONLY_FIELDS` constants to your `Profile` class:

```python
class Profile:
    ADMIN_ONLY_FIELDS = ["email", "last_login"]
    FRIENDS_ONLY_FIELDS = ["last_check_in_location", "favorite_animal"]
```

That way, you can add new fields and authorize access to them without touching
your Polar policy code.
