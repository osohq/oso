---
title: Add Oso to your app
weight: 2
description: |
  An in-depth walkthrough of adding Oso to an example expense application.
aliases:
  - /getting-started/application/index.html
---

# Add To Your Application

This guide covers a little more detail about how to add Oso to your
application.

Whereas in the [Quickstart]({{< relref path="getting-started/quickstart"
lang="java" >}}) we zoomed through an
example of authorization in a simple web server, in this guide we'll show
some more practical examples in the context of a more realistic application.

Our sample expenses application is a Maven project built with Spring Boot.
We are not using anything from Oso that is unique to Spring Boot, and the same patterns we cover here
can be used anywhere.

We highly encourage you to follow along with the code by cloning the example repository
and trying it out. The code can be found here:

[osohq/oso-spring-tutorial](https://github.com/osohq/oso-spring-tutorial)

Our expenses application reads from a sqlite database, and has a few
simple endpoints for returning results. We encourage you to take a look
around before continuing!

## Running The Example

After cloning the example app, make sure to run `mvn install` to download the necessary JARs.

The example comes with some example data, which you can load by running:

```console
$ sqlite3 expenses.db ".read expenses.sql"
```

You can then run the app by running:

```console
$ mvn spring-boot:run
```

## In Your Application

There are two pieces to get started with Oso in your application.
The policy file, and the `oso.isAllowed` call.

The policy file captures the authorization logic you want to apply in
your application, and the `oso.isAllowed` call is used to
enforce that policy in your application.

When starting out, it is reasonable to capture all policy logic in a
single `authorization.polar` file, as we have done here. However, over
time you will want to break it up into multiple files.

Additionally, there are two main places where we want to enforce our
authorization logic: at the request/API layer, and at the data access
layer.

The goal of the former is to restrict which _actions_ a user can take in
your application, e.g. are they allowed to fetch the expenses report via
the `GET /expenses/report` route.

The goal of the latter is to restrict them from viewing data they
shouldn't have access to, e.g. they should not be able to see other
users' data.

### Add Oso

In our sample application, we are storing our policies in the `authorization.polar`
file.

In the application, we need to:

1. Create the Oso instance
2. Load in policy files.
3. [Register application classes](getting-started/policies#application-types)
4. Attach the Oso instance to the application

We have achieved this using the `setupOso` method, in `Application.java`.

```java
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

The first thing we want to add to our application is some simple
authorization to allow some users to only have access to certain routes
if they are logged in.

We can apply apply authorization to **every** incoming request by setting up
a request `Interceptor`, with a `prehandle` function that runs before every request:

```java
@Override
public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler) throws Exception {
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

```python
# authorization.polar

allow(_user, "GET", request: Request) if
    request.getServletPath() = "/";
```

```console
$ curl localhost:5000/
hello Guest
$ curl -H "user: alice@foo.com"  localhost:5000/
hello alice@foo.com
```

But we also have a `/whoami` route that returns a short description of
the current user. We want to make sure only authenticated users can see
this.

We have two different user types here: the `Guest` class and the `User`
class. The latter corresponds to users who have authenticated.

```java
// Guest.java

public class Guest {
  public String toString() {
    return "Guest";
  }
}
```

```java
// User.java

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

We can use [specializer rules](polar-syntax#specialization) to only allow the request
when the actor is an instance of a `User`:

```python
# authorization.polar

allow(_user: User, "GET", request: Request) if
    request.getServletPath() = "/whoami";
```

```console
$ curl -i localhost:5000/whoami
HTTP/1.1 403

$ curl -H "user: alice@foo.com"  localhost:5000/whoami
You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)
```

<!-- {{% callout "Tip" "green" %}}
Interested in understanding more about what is happening here? Check
out the [user types](/guides/user_types) example.
{{% /callout %}} -->

The inputs to the `isAllowed` call are the current user, the HTTP method,
and the HTTP request. This information can often be enough to cover a large
number of uses. For example, if we know that some paths should only
be accessed by certain roles, we can certainly check for this at this point.

In a RESTful application, you can also consider "mapping" authorization
logic from the HTTP path to actions and classes in the application.

For example:

```python
# authorization.polar

allow(user, "GET", http_request) if
    http_request.startsWith("/expenses/")
    and allow(user, "read", Expense);
```

This rule is translating something like `GET /expenses/3` into a check
whether the user should be allowed to "read" the `Expense` class.

However, what if we want to have more fine-grained control? And
authorize access to the precise resource at `/expenses/3`? We'll cover
that in the next section.

### Authorizing Access to Data

In the [Quickstart](quickstart), our main objective was to
determine who could "GET" expenses. Our final policy looked like:

```python
# expenses.polar

allow(actor: String, "GET", expense: Expense) if
    expense.submittedBy = actor;
```

In our expenses sample application, we have something similar,
but we've rewritten the policy to use a new `submitted` predicate in case we want
to change the logic in the future.

```python
# authorization.polar

allow(user: User, "read", expense: Expense) if
    submitted(user, expense);

submitted(user: User, expense: Expense) if
    user.id = expense.userId;
```

To handle authorizing access to data, we've implemented a little helper method
for us to use throughout the application:

```java
// Authorizer.java

public Object authorize(String action, Object resource) {
    try {
      if (!oso.isAllowed(currentUser.get(), action, resource)) {
        throw new ResponseStatusException(HttpStatus.FORBIDDEN, "Oso authorization");
      }
    } catch (OsoException e) {
      throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
    }
    return resource;
}
```

... so authorizing the GET request looks like:

```java
// Controller.java
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

Let's give it a try!

```console
$ curl -i localhost:5000/expenses/2
HTTP/1.1 403

$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

This pattern is pretty convenient. We can easily apply it elsewhere:

```java
// Controller.java

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

```console
$ curl -H "user: alice@foo.com" localhost:5000/organizations/1
Organization(name='Foo Industries', id=1)

$ curl -i -H "user: alice@foo.com" localhost:5000/organizations/2
HTTP/1.1 403
```

Applying this pattern to authorizing data means that the objects we are
passing in to the policy evaluation are already fairly rich objects,
with attributes and methods we can use to make policy decisions. When
starting out, it might be more convenient to apply in the route handler
itself, but try moving it even closer to the data access layer. For
example, if we moved the `authorize` call into the `Expense.lookup`
method, then anywhere our application wants to retrieve an expense, we
are assured that the user does indeed have access to it.

## Your Turn

We currently have a route with no authorization - the submit endpoint.
We have a rule that allows anyone to PUT to the submit endpoint, but we
want to make sure only authorized expenses are submitted.

```python
# authorization.polar

allow_by_path(_user, "PUT", "expenses", ["submit"]);
```

{{% callout "Tip" "green" %}}
The `allow_by_path` rule is a custom rule in our policy that operates
on an actor, action, first url path fragment, and the remaining path
fragment. A `PUT /expenses/submit` request would try to authorize
using the `allow_by_path(actor, "PUT", "expenses", ["submit"])` rule.
See [our policy](https://github.com/osohq/oso-flask-tutorial/blob/ecc39c601057bcfdb952e35da616fe2e1ea00a22/app/authorization.polar#L10) for more detail.
{{% /callout %}}

Right now you can see that anyone can submit an expense:

```console
$ curl -H "user: alice@foo.com" \
  -H "Content-Type: application/json" \
  -X PUT -d '{"amount": 100, "description": "Gummy Bears"}' \
  localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=108)
```

How might we use the `authorize` method from before, to make sure that
we check the user is allowed to `create` this expense?
We would like to do the authorization on the full `Expense` object,
but before it is persisted to the database, so perhaps before this line:

```java {hl_lines=[8]}
// Controller.java

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

```java
    ((Expense) authorizer.authorize("create", expense)).save();
```

This checks the current user is authorized to create the expense.
If this passes, then we can happily move on to the `expense.save()`.
Now, nobody will be able to submit expenses, since we haven't yet
added any rules saying they can.

{{% callout "Add a new rule" "green" %}}
Try editing `authorization.polar` to add a rule saying that
a user can create an expense for which they are assigned as the
submitter of the expense.
{{% /callout %}}

Try editing `authorization.polar` to add a rule saying that a user can
create an expense for which they are assigned as the submitter of the
expense.

Once you have it working, you can test it by verifying as follows:

```console
$ curl -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

$ curl -i -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 2, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
HTTP/1.1 403
```

## Summary

In this guide, we showed a few examples of how to add Oso to a more
realistic application. We added some route-level authorization to
control who is allowed to make requests to certain routes. We also used
a new `authorize` method to make it convenient to add data access
controls to our route handlers.

{{% callout "What's next" "green" %}}

- To explore integrating Oso in your app in more depth continue to [Access Patterns](https://docs.oso.dev/getting-started/application/patterns.html).
- For a deeper introduction to policy syntax, see [Writing Policies](policies).
- For reference on using the Java Oso library, see [Java Authorization Library](reference).
- Clone this example on [GitHub](https://github.com/osohq/oso-spring-tutorial)
  to check it out further.

{{% /callout %}}
