---
weight: 2
title: Django
---

# Django

The oso Django integration adopts Django conventions and provides middleware,
view decorators and ORM integrations to make it easier to use oso with Django.

## Installation

The oso Django integration is available on [PyPI](https://pypi.org/project/django-oso/) and can be installed using
`pip`:

```
$ pip install django-oso
```

## Usage

The `django_oso` django plugin contains a reusable django app that makes
authorization with oso and django easy.  To use, ensure `django_oso` is in
`INSTALLED_APPS`:

```
INSTALLED_APPS = [
    'django_oso',
    ...
]
```

By default, `django_oso` will consider policy files as source code and restart the
server on any changes. To prevent this, add the configuration option:
`OSO_RELOAD_SERVER = False` to your application’s `settings.py` file.

To reload policies on each request in `DEBUG` mode without restarting the
server, you can use the `ReloadPolicyMiddleware` as a complement to the above
configuration change.

### Loading policies

`django_oso` expects policy files to be included in the `policy` directory
of each installed app.  Upon startup, all `.polar` files found in that
directory (or sub-directories) will be loaded using
`oso.Oso.load_file()`.  To load additional files outside of these
directories, call `load_file()` on
`django_oso.oso.Oso`.

### Registering classes & models

Often, authorization rules will be expressed over django models.  Therefore,
`django_oso` will register every model for each installed app upon startup as
a class with oso. The `django.http.HttpRequest` is also registered
under `HttpRequest`.  Django models are referenced in a Polar file using the
syntax `app_name::ModelName`. If an app name contains `.`, for example
`django.contrib.auth`, it will be referenced in oso as
`django::contrib::auth`.

Additional classes can be registered as needed using
`oso.Oso.register_class()` on `django_oso.oso.Oso`.

### Performing authorization

To authorize a request, use the `django_oso.auth.authorize()` function.
It calls
`is_allowed()`, but provides sensible defaults for working with
Django. The actor defaults to `request.user`.  The `action`
defaults to the method of the request.
`resource` must be provided.

`django_oso.auth.authorize()` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.

Here’s a basic example in a route:

```
def get_expense(request, id):
    try:
        expense = Expense.objects.get(pk=id)
    except Expense.DoesNotExist:
        return HttpResponseNotFound()

    authorize(request, expense, action="read")
    return HttpResponse(expense.json())
```

### Requiring authorization on every request

Since `authorize()` is just a function call, it can be
forgotten.  To enforce authorization on every request, use the
`RequireAuthorization()` middleware. Any view that
does not call `authorize()` or
`skip_authorization()` will raise an exception.

### Route authorization

One common usage of `django_oso.auth.authorize()` is to perform authorization
based on the request object. The
`authorize_request()` decorator does this:

```
from django_oso.decorators import authorize_request

@authorize_request
def auth_route(request):
    pass
```

Rules can then be written using request
attributes, like the path:

```
# Allow any actor to make a GET request to "/".
allow(_user: User, "GET", http_request: HttpRequest) if
    http_request.path = "/";
```

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
`RouteAuthorization()` middleware during
initialization.

## Example

Check out the Django integration example app below on GitHub:

** [osohq/oso-django-integration](https://github.com/osohq/oso-django-integration)

## API Reference

### Authorization

### Middleware

### View Decorators

### List endpoint authorization

The oso Django integration includes list filtering support for Django models.

{{< callout "Note" "green" >}}
  These features are in preview and will be stabilized in a future release.
  Please [join our Slack](https://join-slack.osohq.com/) to provide feedback or
  discuss with the engineering team.
{{< /callout >}}

#### Usage

See the list filtering usage guide for more information.

#### API Reference

### Oso


### django_oso.oso.Oso( = <oso.oso.Oso object>)
Singleton `oso.Oso` instance.

Use for loading policy files and registering classes.
