---
weight: 5
title: Flask Authorization Library
aliases:
  - /using/frameworks/flask.html
description: The Oso Flask integration provides request authorization middleware for usage with Flask.
referenceLinks:
    - type: exampleApp
      url: https://github.com/osohq/gitclub
---

# Flask

The Oso Flask integration provides a more convenient interface to Oso for
usage with [Flask](https://flask.palletsprojects.com/).

## Installation

The Oso Flask integration is available on [PyPI](https://pypi.org/project/flask-oso/) and can be installed using
`pip`:

```console
$ pip install flask-oso=={{< version >}}
```

## Usage

### Initialization

The `FlaskOso` class is the entrypoint to the integration.
It must be initialized with the Flask app and Oso:

```python
from flask import Flask
from oso import Oso

app = Flask("app")
oso = Oso()
flask_oso = FlaskOso(app=app, oso=oso)
```

Alternatively, to support the Flask factory pattern, the
`init_app()` method can be used:

```python
from flask import Flask

from oso import Oso
from flask_oso import FlaskOso

oso = Oso()
flask_oso = FlaskOso(oso=oso)

def create_app():
    app = Flask("app")

    # Initialize Oso for this application
    flask_oso.init_app(app)

    return app

app = create_app()
```

This factory function can be a useful place for loading policy files, and
calling configuration functions on `FlaskOso` like
`flask_oso.FlaskOso.require_authorization()`:

```python
def create_app():
    app = Flask("app")

    flask_oso.init_app(app)
    flask_oso.require_authorization(app)

    oso.load_file("authorization.polar")
    oso.register_class(Expense)
```

### Performing authorization

When using the `flask-oso` integration, the primary authorization function is
`flask_oso.FlaskOso.authorize()`. It accepts the same arguments as
`is_allowed()`, but provides sensible defaults for working with
Flask. The actor defaults to `flask.g.current_user` (this can be
customized, see `set_get_actor()`). The `action`
defaults to the method of the current request `flask.request.method`.
`resource` must be provided.

`flask_oso.FlaskOso.authorize()` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.

Here’s a basic example in a route:

```python
@app.route("/<int:id>", methods=["GET"])
def get_expense(id):
    expense = Expense.query.get(id)
    if expense is None:
        raise NotFound()

    flask_oso.authorize(action="read", resource=expense)
    return expense.json()
```

Notice we didn’t need to check the return value of `authorize`. **By default,
a failed authorization will return a \`\`403 Forbidden\`\` response for the current
request.** This can be controlled with
`set_unauthorized_action()`.

#### Working with `LocalProxy` objects

When using a library that exposes the current user (or similar
authorization data) through `LocalProxy` objects, such as [Flask-Login][]'s
`current_user`, you might need to explicitly dereference the proxy
to pass the underlying object to Oso:

[Flask-Login]: https://flask-login.readthedocs.io/en/0.4.1/#flask_login.current_user

```python
from flask_login import current_user

def create_app():
    app = Flask("app")

    flask_oso.init_app(app)
    flask_oso.require_authorization(app)
    # Dereference the current_user LocalProxy
    flask_oso.set_get_actor(lambda: current_user._get_current_object())

    oso.load_file("authorization.polar")
    oso.register_class(User)
    return app
```

By dereferencing the proxy, Oso will use the underlying object when determining
authorization instead of the proxy object.

### Requiring authorization

One downside to calling `flask_oso.FlaskOso.authorize()`
explicitly within route handlers is that the check might be forgotten. To help detect this, the
`flask_oso.FlaskOso.require_authorization()` option can be enabled during
initialization. This will cause an `oso.OsoError` to be raised if
a call to `flask_oso.FlaskOso.authorize()` **is not** made during the
processing of a request.

Sometimes a route will not need authorization. To prevent this route from
causing an authorization error, call
`flask_oso.FlaskOso.skip_authorization()` during request processing:

```python
oso = Oso()
flask_oso = FlaskOso()

def create_app():
    app = Flask("app")

    flask_oso.init_app(app)
    flask_oso.require_authorization(app)

    oso.load_file("authorization.polar")

    return app

app = create_app()

@app.route("/about")
def about():
    flask_oso.skip_authorization()
    return "about us"
```

### Using decorators

Some developers may prefer a decorator-based API for performing authorization.
`flask_oso` provides two decorators:

`flask_oso.skip_authorization()` marks a route as not requiring
authorization. It is the decorator version of
`flask_oso.FlaskOso.skip_authorization()`.

`flask_oso.authorize()` decorates a route and calls
`flask_oso.FlaskOso.authorize()` before the route body is entered. For
example:

```python
from flask_oso import authorize

@authorize(resource="get_user")
@app.route("/user")
def get_user():
    return "current user"
```

This decorator can be used if the resource is known before entering the request
body.

### Route authorization

One common usage of `flask_oso.authorize()` is to perform authorization
based on the Flask request object:

```python
from flask import request

@flask_oso.authorize(resource=request)
@app.route("/")
def route():
    return "authorized"
```

A policy can then be written controlling authorization based on request
attributes, like the path:

```polar
# Allow any actor to make a GET request to "/".
allow(_actor, "GET", _resource: Request{path: "/"});
```

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
`perform_route_authorization()` method during
initialization.

## Example

Check out the Flask integration example app on GitHub:
[osohq/oso-flask-integration](https://github.com/osohq/oso-flask-integration).

## API Reference

The [Flask API reference]({{% apiLink "reference/api/flask.html" %}})
is automatically generated from the Oso Flask library source files.
