---
weight: 2
title: Flask
---

# Flask

The oso Flask integration provides a more convenient interface to oso for
usage with [Flask](https://flask.palletsprojects.com/).

## Installation

The oso Flask integration is available on [PyPI](https://pypi.org/project/flask-oso/) and can be installed using
`pip`:

```
$ pip install flask-oso
```

## Usage

### Initialization

The `FlaskOso` class is the entrypoint to the integration.
It must be initialized with the Flask app and oso:

```
from flask import Flask
from oso import Oso

app = Flask("app")
oso = Oso()
flask_oso = FlaskOso(app=app, oso=oso)
```

Alternatively, to support the Flask factory pattern, the
`init_app()` method can be used:

```
from flask import Flask

from oso import Oso
from flask_oso import FlaskOso

oso = Oso()
flask_oso = FlaskOso(oso=oso)

def create_app():
    app = Flask("app")

    # Initialize oso for this application
    flask_oso.init_app(app)

    return app

app = create_app()
```

This factory function can be a useful place for loading policy files, and
calling configuration functions on `FlaskOso` like
`flask_oso.FlaskOso.require_authorization()`:

```
def create_app():
    app = Flask("app")

    flask_oso.init_app(app)
    flask_oso.require_authorization(app)

    oso.load_file("authorization.polar")
    oso.register_class(Expense)
```

### Performing authorization

When using the `flask-oso` integration, the primary authorization function is
`flask_oso.FlaskOso.authorize()`.  It accepts the same arguments as
`is_allowed()`, but provides sensible defaults for working with
Flask. The actor defaults to `flask.g.current_user` (this can be
customized, see `set_get_actor()`).  The `action`
defaults to the method of the current request `flask.request.method`.
`resource` must be provided.

`flask_oso.FlaskOso.authorize()` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.

Here’s a basic example in a route:

```
@app.route("/<int:id>", methods=["GET"])
def get_expense(id):
    expense = Expense.query.get(id)
    if expense is None:
        raise NotFound()

    flask_oso.authorize(action="read", resource=expense)
    return expense.json()
```

Notice we didn’t need to check the return value of `authorize`.  **By default,
a failed authorization will return a \`\`403 Forbidden\`\` response for the current
request.** This can be controlled with
`set_unauthorized_action()`.

### Requiring authorization

One downside to calling `flask_oso.FlaskOso.authorize()`
explicitly within route handlers is that the check might be forgotten.  To help detect this, the
`flask_oso.FlaskOso.require_authorization()` option can be enabled during
initialization. This will cause an `oso.OsoError` to be raised if
a call to `flask_oso.FlaskOso.authorize()` **is not** made during the
processing of a request.

Sometimes a route will not need authorization.  To prevent this route from
causing an authorization error, call
`flask_oso.FlaskOso.skip_authorization()` during request processing:

```
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
authorization.  It is the decorator version of
`flask_oso.FlaskOso.skip_authorization()`.

`flask_oso.authorize()` decorates a route and calls
`flask_oso.FlaskOso.authorize()` before the route body is entered. For
example:

```
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

```
from flask import request

@flask_oso.authorize(resource=request)
@app.route("/")
def route():
    return "authorized"
```

A policy can then be written controlling authorization based on request
attributes, like the path:

```
# Allow any actor to make a GET request to "/".
allow(_actor, action: "GET", resource: Request{path: "/"});
```

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
`perform_route_authorization()` method during
initialization.

## Example

Check out the Flask integration example app below on GitHub:

** [osohq/oso-flask-integration](https://github.com/osohq/oso-flask-integration)

## API Reference


### class flask_oso.FlaskOso(oso=None, app=None)
oso flask plugin

This plugin must be initialized with a flask app, either using the
`app` parameter in the constructor, or by calling `init_app()` after
construction.

The plugin must be initialized with an `oso.Oso` instance before
use, either by passing one to the constructor or calling
`set_oso()`.

**Authorization**


* `FlaskOso.authorize()`: Check whether an actor, action and resource is
authorized. Integrates with flask to provide defaults for actor & action.

**Configuration**


* `require_authorization()`: Require at least one
`FlaskOso.authorize()` call for every request.


* `set_get_actor()`: Override how oso determines the actor
associated with a request if none is provided to `FlaskOso.authorize()`.


* `set_unauthorized_action()`: Control how `FlaskOso.authorize()`
handles an unauthorized request.


* `perform_route_authorization()`:
Call authorize(resource=flask.request) before every request.


#### authorize(resource, \*, actor=None, action=None)
Check whether the current request should be allowed.

Calls `oso.Oso.is_allowed()` to check authorization. If a request
is unauthorized, raises a `werkzeug.exceptions.Forbidden`
exception.  This behavior can be controlled with
`set_unauthorized_action()`.


* **Parameters**

    
    * **actor** – The actor to authorize. Defaults to `flask.g.current_user`.
    Use `set_get_actor()` to override.


    * **action** – The action to authorize. Defaults to
    `flask.request.method`.


    * **resource** – The resource to authorize.  The flask request object
    (`flask.request`) can be passed to authorize a
    request based on route path or other request properties.


See also: `flask_oso.authorize()` for a route decorator version.


#### init_app(app)
Initialize `app` for use with this instance of `FlaskOso`.

Must be called if `app` isn’t provided to the constructor.


#### perform_route_authorization(app=None)
Perform route authorization before every request.

Route authorization will call `oso.Oso.is_allowed()` with the
current request (from `flask.request`) as the resource and the method
(from `flask.request.method`) as the action.


* **Parameters**

    **app** – The app to require authorization for. Can be omitted if
    the `app` parameter was used in the `FlaskOso`
    constructor.



#### require_authorization(app=None)
Enforce authorization on every request to `app`.


* **Parameters**

    **app** – The app to require authorization for. Can be omitted if
    the `app` parameter was used in the `FlaskOso`
    constructor.


If `FlaskOso.authorize()` is not called during the request processing,
raises an `oso.OsoError`.

Call `FlaskOso.skip_authorization()` to skip this check for a particular
request.


#### set_get_actor(func)
Provide a function that oso will use to get the current actor.


* **Parameters**

    **func** – A function to call with no parameters to get the actor if
    it is not provided to `FlaskOso.authorize()`. The return value
    is used as the actor.



#### set_oso(oso)
Set the oso instance to use for authorization

Must be called if `oso` is not provided to the constructor.


#### set_unauthorized_action(func)
Set a function that will be called to handle an authorization failure.

The default behavior is to raise a Forbidden exception, returning a 403
response.


* **Parameters**

    **func** – A function to call with no parameters when a request is
    not authorized.



#### skip_authorization(reason=None)
Opt-out of authorization for the current request.

Will prevent `require_authorization` from causing an error.

See also: `flask_oso.skip_authorization()` for a route decorator version.


### flask_oso.authorize(func=None, resource=None, actor=None, action=None)
Flask route decorator.  Calls `FlaskOso.authorize()` before the route.

Parameters are the same as `FlaskOso.authorize()`.

**WARNING**: This decorator must come **after** the `route` decorator as shown below, otherwise authorization will not be
checked.

For example:

```
@app.route("/")
@authorize(resource=flask.request)
def route():
    return "authorized"
```


### flask_oso.skip_authorization(func=None, reason=None)
Decorator to mark route as not requiring authorization.

**WARNING**: This decorator must come after the `route` decorator.

Causes use in conjunction with `FlaskOso.require_authorization()` to
silence errors on routes that do not need to be authorized.
