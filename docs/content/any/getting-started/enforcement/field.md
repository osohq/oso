---
title: "Field-level Enforcement"
weight: 30
any: true
# draft: True
---

{{% callout "Note: Preview API" %}}
  This is a preview API, meaning that it is not yet officially released. You may
  find other docs that conflict with the guidance here, so proceed at your own
  risk! If you have any questions, don't hesitate to [reach out to us on
  Slack](https://join-slack.osohq.com). We're here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Field-level Enforcement

Perhaps you're creating a user profile page, and in your app, only a user's
friends can access their last check-in location. To enforce this type of
authorization, it often makes sense to use **"field-level" enforcement**, by
explicitly allowing access to certain fields on your domain objects.

Field-level authorization gives you fine-grained control over who can access
exactly what bits of information. In polar, you can write field-level rules like
this:

```polar
allow_field(user, "read", profile, "last_check_in_location") if
    profile.user in user.friends;
```

Notice that an `allow_field` rule is just like an `allow` rule, except that it
takes an additional argument: the field name.

To enforce field-level authorization in your app, you use the `authorize_field`
method provided by Oso enforcers.

{{% minicallout %}}
**Note**: You'll need an Oso enforcer to use the `authorize_field` method. Check out
[how to build one](enforcer.html).
{{% /minicallout %}}

```python
def get_last_check_in_location(profile, current_user):
    oso.authorize_field(current_user, "read", profile, "last_check_in_location");
    return profile.last_check_in.location
```

Like `authorize`, `authorize_field` will raise an `AuthorizationError` when the
user is not allowed to perform the given action. This is an error that you
should handle globally in your app. You can read more details about this on the
[Resource-level Enforcement Guide](resource.html#authorization-failure).

Sometimes it is helpful to get
_all_ fields that a user can access, and for this there is a separate method
called `authorized_fields`:

```python
# Serialize only the fields of profile that the current user is allowed to read
def serialize_profile(profile, current_user):
    fields = oso.authorized_fields(current_user, "read", profile)
    return { field: profile[field] for key in fields }
```

The `authorized_fields` method can be used to send only the fields that the user
is explicitly allowed to read, or can similarly be used to filter _incoming_
parameters from a user for a call to, say, an `update` method. In that case, you
might use an `"update"` action in the call to `authorized_fields`:

```python
# Filter update_params by the fields on profile that the user can update
def filter_update_params(profile, raw_update_params, current_user):
    fields = oso.authorized_fields(current_user, "update", profile)
    return { field: raw_update_params[field] for key in fields }
```
