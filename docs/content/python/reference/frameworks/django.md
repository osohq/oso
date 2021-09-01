---
weight: 4
title: Django Authorization Library
aliases:
  - /using/frameworks/django.html
description: The Oso Django integration provides request middleware and ORM integrations for data filtering.
referenceLinks:
    - type: exampleApp
      url: https://github.com/osohq/oso-github-django
---

# Django

The Oso Django integration adopts Django conventions and provides middleware,
view decorators and ORM integrations to make it easier to use Oso with Django.

## Installation

The Oso Django integration is available on [PyPI](https://pypi.org/project/django-oso/) and can be installed using
`pip`:

```console
$ pip install django-oso=={{< version >}}
```

## Usage

The `django_oso` Django plugin contains a reusable Django app that makes
authorization with Oso and Django easy. To use, ensure `django_oso` is in
`INSTALLED_APPS`:

```python
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
of each installed app. Upon startup, all `.polar` files found in that
directory (or sub-directories) will be loaded using
`oso.Oso.load_file()`. To load additional files outside of these
directories, call `load_file()` on
`django_oso.oso.Oso`.

### Registering classes & models

Often, authorization rules will be expressed over Django models. Therefore,
`django_oso` will register every model for each installed app upon startup as
a class with Oso. The `django.http.HttpRequest` is also registered
under `HttpRequest`. Django models are referenced in a Polar file using the
syntax `app_name::ModelName`. If an app name contains `.`, for example
`django.contrib.auth`, it will be referenced in Oso as
`django::contrib::auth`.

Additional classes can be registered as needed using
`oso.Oso.register_class()` on `django_oso.oso.Oso`.

### Performing authorization

To authorize a request, use the `django_oso.auth.authorize()` function.
It calls
`is_allowed()`, but provides sensible defaults for working with
Django. The actor defaults to `request.user`. The `action`
defaults to the method of the request.
`resource` must be provided.

`django_oso.auth.authorize()` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.

Here’s a basic example in a route:

```python
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
forgotten. To enforce authorization on every request, use the
`RequireAuthorization()` middleware. Any view that
does not call `authorize()` or
`skip_authorization()` will raise an exception.

### Route authorization

One common usage of `django_oso.auth.authorize()` is to perform authorization
based on the request object. The
`authorize_request()` decorator does this:

```python
from django_oso.decorators import authorize_request

@authorize_request
def auth_route(request):
    pass
```

Rules can then be written using request
attributes, like the path:

```polar
# Allow any actor to make a GET request to "/".
allow(_user: User, "GET", http_request: HttpRequest) if
    http_request.path = "/";
```

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
`RouteAuthorization()` middleware during
initialization.

## Example

Check out [the Django integration example
app](https://github.com/osohq/oso-django-integration) on GitHub.

## API Reference

The [Django API reference]({{% apiLink "reference/api/django.html" %}})
is automatically generated from the Oso Django library source files.
