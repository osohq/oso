<!-- TODO: TBD where to add -->

# Add To Your Application

This guide covers a little more detail about how to add Oso to your application.

Whereas in the Quickstart we zoomed through an example
of authorization in a simple web server, in this guide we’ll show some more
practical examples in the context of a more realistic application.

Python

Our sample expenses application is built with Flask, but we are not using
anything from Oso that is unique to Flask, and the same patterns we cover here
can be used anywhere.

We highly encourage you to follow along with the code by cloning the example repository
and trying it out. The code can be found here:

** [osohq/oso-flask-tutorial](https://github.com/osohq/oso-flask-tutorial)

Java

Our sample expenses application is a Maven project built with Spring Boot.
We are not using anything from Oso that is unique to Spring Boot, and the same patterns we cover here
can be used anywhere.

We highly encourage you to follow along with the code by cloning the example repository
and trying it out. The code can be found here:

** [osohq/oso-spring-tutorial](https://github.com/osohq/oso-spring-tutorial)

Our expenses application reads from a SQLite database, and has a few simple endpoints for returning
results. We encourage you to take a look around before continuing!

## Running The Example

Python

The application has a few requirements, including Flask and, of course, Oso.
We recommend installing these within a virtual environment:

```
$ git clone https://github.com/osohq/oso-flask-tutorial/
$ cd oso-flask-tutorial/
$ python3 -m venv venv
$ source venv/bin/activate
$ pip3 install -r requirements.txt
$ FLASK_ENV=development flask run --extra-files app/authorization.polar
```

The example comes with some example data, which you can load with:

```
$ sqlite3 expenses.db ".read expenses.sql"
```

Java

After cloning the example app, make sure to run `mvn install` to download the necessary JARs.

The example comes with some example data, which you can load by calling:

```
$ sqlite3 expenses.db ".read expenses.sql"
```

You can then run the app by calling

```
$ mvn spring-boot:run
```

## In Your Application

Python

There are two pieces to get started with Oso in your application.
The policy file, and the `oso.is_allowed` call.

The policy file captures the authorization logic you want to apply in
your application, and the `oso.is_allowed` call is used to
enforce that policy in your application.

Java

There are two pieces to get started with Oso in your application.
The policy file, and the `oso.isAllowed` call.

The policy file captures the authorization logic you want to apply in
your application, and the `oso.isAllowed` call is used to
enforce that policy in your application.

When starting out, it is reasonable to capture all policy logic in
a single `authorization.polar` file, as we have done here.
However, over time you will want to break it up into multiple
files.

Additionally, there are two main places where we want to enforce our
authorization logic: at the request/API layer, and at the data access
layer.

The goal of the former is to restrict which *actions* a user can take in
your application, e.g. are they allowed to fetch the
expenses report via the `GET /expenses/report` route.

The goal of the latter is to restrict them from viewing data they shouldn’t have
access to, e.g. they should not be able to see other users’ data.

### Add Oso

Python

In our sample application, we are storing our policies in the `authorization.polar`
file, and all of the authorization in the application is managed through the
`authorization.py` file.

In the application, we need to:


1. Create the Oso instance


2. Load in policy files.


3. Register application classes


4. Attach the Oso instance to the application

We have achieved this using the `init_oso` method:

```
def init_oso(app):
    from .expense import Expense
    from .organization import Organization
    from .user import Actor, Guest, User

    oso = Oso()
    oso.register_class(Actor)
    oso.register_class(Guest)
    oso.register_class(User)
    oso.register_class(Expense)
    oso.register_class(Organization)
    oso.register_class(Request)

    for policy in app.config.get("OSO_POLICIES", []):
        oso.load_file(policy)

    app.oso = oso
```

Java

In our sample application, we are storing our policies in the `authorization.polar`
file.

In the application, we need to:


1. Create the Oso instance


2. Load in policy files.


3. Register application classes


4. Attach the Oso instance to the application

We have achieved this using the `setupOso` method, in `Application.java`.

```

  @Bean
  public Oso setupOso() throws IOException, Exceptions.OsoException {
    Oso oso = new Oso();
    oso.registerClass(User.class, "User");
    oso.registerClass(Expense.class, "Expense");
    oso.registerClass(Organization.class, "Organization");
    oso.registerClass(HttpServletRequest.class, "Request");
    oso.loadFile("src/main/oso/authorization.polar");
    return oso;
  }
```

We can now access this `oso` instance anywhere in our application, and specify
which policy files are loaded in the application configuration.

### Authorizing Routes

The first thing we want to add to our application is some simple authorization
to allow some users to only have access to certain routes if they are logged in.

Python

We can apply apply authorization to **every** incoming request by setting up
a middleware function that runs before every request using `before_app_request`:

```
@bp.before_app_request
def authorize_request():
    """Authorize the incoming request"""
    r = request._get_current_object()
    if not current_app.oso.is_allowed(g.current_user, r.method, r):
        return Forbidden("Not Authorized!")
```

Java

We can apply apply authorization to **every** incoming request by setting up
a request `Interceptor`, with a `prehandle` function that runs before every request:

```
  public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
      throws Exception {
    try {
      setCurrentUser(request);

      // Authorize the incoming request
      if (!oso.isAllowed(currentUser.get(), request.getMethod(), request)) {
        throw new ResponseStatusException(HttpStatus.FORBIDDEN, "oso authorization: unauthorized");
      }
    } catch (SQLException e) {
      throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "User not found", e);
    }
    return true;
  }
```

Now that this is in place, we can write a simple policy to allow anyone
to call our index route, and see the hello message:

```
allow(_user, "GET", request: Request) if
    request.path = "/";
```

```
$ curl localhost:5000/
hello Guest
$ curl -H "user: alice@foo.com"  localhost:5000/
hello alice@foo.com
```

But we also have a `/whoami` route that returns a short description
of the current user. We want to make sure only authenticated users can
see this.

We have two different user types here: the `Guest` class and the `User` class.
The latter corresponds to users who have authenticated.

Python

```
class Guest(Actor):
    """Anonymous user."""
```

```
@dataclass
class User(Actor):
    """Logged in user. Has an email address."""

    id: int
    email: str
    title: str
    location_id: int
    organization_id: int
    manager_id: int
```

Java

```
public class Guest {
  public String toString() {
    return "Guest";
  }
}
```

```
public class User {
  public Integer id, locationId, organizationId, managerId;
  public String email, title;

  public User(
      Integer id,
      Integer locationId,
      Integer organizationId,
      Integer managerId,
      String email,
      String title) {
    this.id = id;
    this.locationId = locationId;
    this.organizationId = organizationId;
    this.managerId = managerId;
    this.email = email;
    this.title = title;
  }
```

We can use specializer rules to only allow the request
when the actor is an instance of a `User`:

```
allow(_user: User, "GET", request: Request) if
    request.path = "/whoami";
```

Python

```
$ curl localhost:5000/whoami
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>

$ curl -H "user: alice@foo.com"  localhost:5000/whoami
You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)
```

Java

```
$ curl -i localhost:5000/whoami
HTTP/1.1 403

$ curl -H "user: alice@foo.com"  localhost:5000/whoami
You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)
```

Python

The inputs to the `is_allowed` call are the current user, the HTTP method,
and the HTTP request. This information can often be enough to cover a large
number of uses. For example, if we know that some paths should only
be accessed by certain roles, we can certainly check for this at this point.

Java

The inputs to the `isAllowed` call are the current user, the HTTP method,
and the HTTP request. This information can often be enough to cover a large
number of uses. For example, if we know that some paths should only
be accessed by certain roles, we can certainly check for this at this point.

In a RESTful application, you can also consider “mapping” authorization
logic from the HTTP path to actions and classes in the application.

For example:

Python

```
    allow(user, "GET", http_request) if
        http_request.startswith("/expenses/")
        and allow(user, "read", Expense);
```

Java

```
    allow(user, "GET", http_request) if
        http_request.startsWith("/expenses/")
        and allow(user, "read", Expense);
```

This rule is translating something like `GET /expenses/3` into a check
whether the user should be allowed to “read” the `Expense` class.

However, what if we want to have more fine-grained control? And authorize access
to the precise resource at `/expenses/3`? We’ll cover that in the next
section.

### Authorizing Access to Data

Python

In the Quickstart, our main objective was to
determine who could “GET” expenses. Our final policy looked like:

```
allow(actor: String, "GET", expense: Expense) if
    expense.submitted_by = actor;
```

In our expenses sample application, we have something similar,
but we’ve rewritten the policy to use a new `submitted` predicate in case we want
to change the logic in the future.

```
allow(user: User, "read", expense: Expense) if
    submitted(user, expense);

submitted(user: User, expense: Expense) if
    user.id = expense.user_id;
```

To handle authorizing access to data, we’ve implemented a little helper method
for us to use throughout the application:

```
def authorize(action, resource):
    """Authorize whether the current user can perform `action` on `resource`"""
    if current_app.oso.is_allowed(g.current_user, action, resource):
        return resource
    else:
        raise Forbidden("Not Authorized!")
```

… so authorizing the GET request looks like:

```
def get_expense(id):
    expense = Expense.lookup(id)
    return str(authorize("read", expense))
```

Let’s give it a try!

```
$ curl localhost:5000/expenses/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>

$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

This pattern is pretty convenient. We can easily apply it elsewhere:

```
def get_organization(id):
    organization = Organization.lookup(id)
    return str(authorize("read", organization))
```

```
$ curl -H "user: alice@foo.com" localhost:5000/organizations/1
Organization(name='Foo Industries', id=1)

$ curl -H "user: alice@foo.com" localhost:5000/organizations/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

Java

In the Quickstart, our main objective was to
determine who could “GET” expenses. Our final policy looked like:

```
allow(actor: String, "GET", expense: Expense) if
    expense.submittedBy = actor;
```

In our expenses sample application, we have something similar,
but we’ve rewritten the policy to use a new `submitted` predicate in case we want
to change the logic in the future.

```
allow(user: User, "read", expense: Expense) if
    submitted(user, expense);

submitted(user: User, expense: Expense) if
    user.id = expense.userId;
```

To handle authorizing access to data, we’ve implemented a little helper method
for us to use throughout the application:

```
  public Object authorize(String action, Object resource) {
    try {
      if (!oso.isAllowed(currentUser.get(), action, resource)) {
        throw new ResponseStatusException(HttpStatus.FORBIDDEN, "oso authorization");
      }
    } catch (OsoException e) {
      throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
    }
    return resource;
  }
```

… so authorizing the GET request looks like:

```
  @GetMapping("/expenses/{id}")
  public String getExpense(@PathVariable(name = "id") int id) {
    try {
      Expense e = Expense.lookup(id);
      return authorizer.authorize("read", e).toString();
    } catch (SQLException e) {
      throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Expense not found", e);
    }
  }
```

Let’s give it a try!

```
$ curl -i localhost:5000/expenses/2
HTTP/1.1 403

$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

This pattern is pretty convenient. We can easily apply it elsewhere:

```
  @GetMapping("/organizations/{id}")
  public String getOrganization(@PathVariable(name = "id") int id) {
    try {
      Organization org = Organization.lookup(id);
      return authorizer.authorize("read", org).toString();
    } catch (SQLException e) {
      throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Organization not found", e);
    }
  }
```

```
$ curl -H "user: alice@foo.com" localhost:5000/organizations/1
Organization(name='Foo Industries', id=1)

$ curl -i -H "user: alice@foo.com" localhost:5000/organizations/2
HTTP/1.1 403
```

Applying this pattern to authorizing data means that the objects we are passing
in to the policy evaluation are already fairly rich objects, with attributes and
methods we can use to make policy decisions. When starting out, it might be more
convenient to apply in the route handler itself, but try moving it even closer
to the data access layer. For example, if we moved the `authorize` call into
the `Expense.lookup` method, then anywhere our application wants to retrieve
an expense, we are assured that the user does indeed have access to it.

## Your Turn

We currently have a route with no authorization - the submit endpoint.
We have a rule that allows anyone to PUT to the submit endpoint, but
we want to make sure only authorized expenses are submitted.

```
allow_by_path(_user, "PUT", "expenses", ["submit"]);
```

Python

Right now you can see that anyone can submit an expense:

```
$ curl -H "user: alice@foo.com" -X PUT -d '{"amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=108)
```

How might we use the `authorize` method from before, to make sure that
we check the user is allowed to `create` this expense?
We would like to do the authorization on the full `Expense` object,
but before it is persisted to the database, so perhaps between these two
lines:

```
def submit_expense():
    expense_data = request.get_json(force=True)
    if not expense_data:
        raise BadRequest()
    # if no user id supplied, assume it is for the current user
    expense_data.setdefault("user_id", g.current_user.id)
    expense = Expense(**expense_data)
    expense.save()
    return str(expense)
```

We could change the first highlighted line to:

```
expense = authorize("create", Expense(**expense_data))
```

This checks the current user is authorized to create the expense.
If this passes, then we can happily move on to the `expense.save()`.

Now, nobody will be able to submit expenses, since we haven’t yet
added any rules saying they can.

Java

Right now you can see that anyone can submit an expense:

```
$ curl -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=108)
```

How might we use the `authorize` method from before, to make sure that
we check the user is allowed to `create` this expense?
We would like to do the authorization on the full `Expense` object,
but before it is persisted to the database, so perhaps before this line:

```
  @PutMapping("/expenses/submit")
  public String submitExpense(@RequestBody Expense expense) {
    try {
      User user = (User) currentUser.get();
      if (expense.userId == 0) expense.userId = user.id;
      expense.save();
      return expense.toString();
    } catch (SQLException e) {
      throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "failed to save expense", e);
    }
  }
```

We could change the highlighted line to:

```
((Expense) authorizer.authorize("create", expense)).save();
```

This checks the current user is authorized to create the expense.
If this passes, then we can happily move on to the `expense.save()`.

Now, nobody will be able to submit expenses, since we haven’t yet
added any rules saying they can.

Once you have it working, you can test it by verifying as follows:

Python

```
$ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

$ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 2, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

Java

```
$ curl -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

$ curl -i -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 2, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
HTTP/1.1 403
```

## Summary

In this guide, we showed a few examples of how to add Oso to a more realistic
application. We added some route-level authorization to control who is allowed
to make requests to certain routes. We also used a new `authorize` method to
make it convenient to add data access controls to our route handlers.
