---
date: '2021-01-07T02:46:33.217Z'
docname: getting-started/list-filtering/sqlalchemy
images: {}
path: /getting-started-list-filtering-sqlalchemy
title: Filter collections with SQLAlchemy
aliases:
    - /getting-started/list-filtering/sqlalchemy.html
description: Learn to use Oso's SQLAlchemy integration to filter collections of data.
---

# Filter collections with SQLAlchemy

The `sqlalchemy-oso` library can enforce policies over SQLAlchemy models. This
allows policies to control access to collections of objects without needing to
authorize each object individually.

## Installation

The Oso SQLAlchemy integration is available on
[PyPI](https://pypi.org/project/sqlalchemy-oso/) and can be installed using
`pip`:

```console
$ pip install sqlalchemy-oso
```

{{% callout "Early access to the next version of sqlalchemy-oso" "blue" %}}

We just released early access to the next version of our SQLAlchemy
library, which will include improved support for common authorization
models, like role based access control.

[Check it out here!](/new-roles)

{{% /callout %}}

## Usage

`sqlalchemy-oso` works over your existing SQLAlchemy ORM models without
modification.

To get started, we need to:

1. Make Oso aware of our SQLAlchemy model types so that we can write policies
   over them.
2. Create a SQLAlchemy
   [Session](https://docs.sqlalchemy.org/en/13/orm/session_api.html#sqlalchemy.orm.session.Session)
   that uses Oso to authorize access to data.

### Register models with Oso

`sqlalchemy_oso.register_models()` registers all models that descend from a
declarative base class as types that are available in the policy.

Alternatively, the `oso.Oso.register_class()` method can be called on each
SQLAlchemy model that you want to reference in your policy.

### Create a SQLAlchemy Session that uses Oso

Oso performs authorization by integrating with SQLAlchemy sessions. Use the
`sqlalchemy_oso.authorized_sessionmaker()` session factory instead of the
default SQLAlchemy `sessionmaker`. Every query made using sessions from the
`authorized_sessionmaker()` factory will have authorization applied.

Before executing a query, Oso consults the policy and obtains a list of
conditions that must be met for a model to be authorized. These conditions are
translated into SQLAlchemy expressions and applied to the query before
retrieving objects from the database.

## Using with Flask

`sqlalchemy-oso` has built in support for the popular `flask_sqlalchemy`
library. <!-- TODO(gj): bring back when API docs are up:
See `sqlalchemy_oso.flask.AuthorizedSQLAlchemy`. -->

## Example

Let’s look at an example usage of this library. Our example is a social media
app that allows users to create posts. There is a `Post` model and a `User`
model:

{{< literalInclude path="examples/data_access/sqlalchemy/sqlalchemy_example/models.py" >}}

Now, we’ll write a policy over these models. Our policy contains the following
rules:

1. A user can read any public post.
2. A user can read their own private posts.
3. A user can read private posts for users they manage (defined through the
   `user.manages` relationship).
4. A user can read all other users.

{{< literalInclude path="examples/data_access/sqlalchemy/sqlalchemy_example/policy.polar" >}}

{{% callout "Note" "blue" %}}
  The SQLAlchemy integration is deny by default. The final rule for `User` is
  needed to allow access to user objects for any user.

  If a query is made for a model that does not have an explicit rule in the
  policy, no results will be returned.
{{% /callout %}}

These rules are written over single model objects.

### Trying it out

Let’s test out the policy in a REPL.

First, import `sqlalchemy`, `oso`, and `sqlalchemy_oso`:

```python
>>> from sqlalchemy import create_engine
>>> from sqlalchemy.orm import Session
>>> from oso import Oso
>>> from sqlalchemy_oso import authorized_sessionmaker, register_models
>>> from sqlalchemy_example.models import Model, User, Post
```

Then, setup `oso` and register our models.

```python
>>> oso = Oso()
>>> register_models(oso, Model)
>>> oso.load_file("sqlalchemy_example/policy.polar")
```

Next, setup some test data…

```python
>>> user = User(username='user')
>>> manager = User(username='manager', manages=[user])
>>> public_user_post = Post(contents='public_user_post',
...                         access_level='public',
...                         created_by=user)
>>> private_user_post = Post(contents='private_user_post',
...                          access_level='private',
...                          created_by=user)
>>> private_manager_post = Post(contents='private_manager_post',
...                             access_level='private',
...                             created_by=manager)
>>> public_manager_post = Post(contents='public_manager_post',
...                            access_level='public',
...                            created_by=manager)
```

… and load that data into SQLAlchemy:

```python
>>> engine = create_engine('sqlite:///:memory:')
>>> Model.metadata.create_all(engine)
>>> fixture_session = Session(bind=engine)
>>> fixture_session.add_all([
...     user, manager, public_user_post, private_user_post, private_manager_post,
...     public_manager_post])
>>> fixture_session.commit()
```

### Authorizing a user’s posts

Now that we’ve setup some test data, let’s use **oso** to authorize `Post`s
that `User(username="user")` can see.

We’ll start by making an `authorized_sessionmaker()`:

```python
>>> AuthorizedSession = authorized_sessionmaker(bind=engine,
...                                             get_oso=lambda: oso,
...                                             get_user=lambda: user,
...                                             get_action=lambda: "read")
>>> session = AuthorizedSession()
```

Then, issue a query for all posts:

```python
>>> posts = session.query(Post).all()
>>> [p.contents for p in posts]
['public_user_post', 'private_user_post', 'public_manager_post']
```

Since we used `authorized_sessionmaker()`, the query only returned authorized
posts based on the policy.

`User(username="user")` can see their own public and private posts and public
posts made by other users.

### Authorizing a manager’s posts

Now we’ll authorize access to `User(username="manager")`'s `Post`s. We create a
new authorized session with user set to `manager`:

```python
>>> AuthorizedSession = authorized_sessionmaker(bind=engine,
...                                             get_oso=lambda: oso,
...                                             get_user=lambda: manager,
...                                             get_action=lambda: "read")
>>> manager_session = AuthorizedSession()
```

{{% callout "Note" "blue" %}}
  In a real application, `get_user` would be a function returning the current
  user based on the current request context. For example, in Flask this might
  be `lambda: flask.g.current_user` or some other proxy object.
{{% /callout %}}

And issue the same query as before…

```python
>>> posts = manager_session.query(Post).all()
>>> [p.contents for p in posts]
['public_user_post', 'private_user_post', 'private_manager_post', 'public_manager_post']
```

This time, the query returned four posts! Since the `manager` user manages
`user`, the private post of user is also authorized (based on our third rule
above).

```python
>>> manager.manages[0].username
'user'
```

This full example is available on
[GitHub](https://github.com/osohq/oso/tree/main/docs/examples/data_access/sqlalchemy).

## How Oso authorizes SQLAlchemy Data

As you can see from the above example, the SQLAlchemy Oso integration allows
regular SQLAlchemy queries to be executed with authorization applied.

Before compiling a SQLAlchemy query, the entities in the query are authorized
with Oso. Oso returns authorization decisions for each entity that indicate
what constraints must be met for the entity to be authorized. These constraints
are then translated into filters on the SQLAlchemy query object.

For example, our above policy has the following code:

```polar
allow(user: User, "read", post: Post) if
    post.access_level = "private" and
    post.created_by = user;
```

The Oso library converts the constraints on `Post` expressed in this policy
into a SQLAlchemy query like:

```python
session.query(Post)
    .filter(Post.access_level == "private" & Post.created_by == user)
```

This translation makes the policy an effective abstraction for expressing
authorization logic over collections.

## Limitations

This feature is still under active development. Not all policies that work in a
non-partial setting will currently work with partials. More policies will be
supported as we continue working on this feature. The SQLAlchemy adapter is
ready for evaluation and testing. However, we recommend getting in touch with
us on [Slack](https://join-slack.osohq.com/) before using it in production.

There are some operators and features that do not currently work with the
SQLAlchemy adapter when used **anywhere in the policy**:

* The `cut` operator
* Rules that rely on ordered execution based on class inheritance
* Negated queries using the `not` operator that contain a `matches` operation
  within the negation or call a rule containing a specializer. For example:

    ```polar
    # Not supported.
    allow(actor, action, resource) if
        not resource matches User;

    # Also not supported.
    is_user(user: User);
    allow(actor, action, resource) if
        not is_user(resource);
    ```

Some operations cannot be performed on **resources** in `allow` rules used with
the SQLAlchemy adapter. These operations can still be used on the actor or
action:

* Application method calls
* Arithmetic operators

<!-- TODO(gj): bring back when API docs are up.

## API Reference

See SQLAlchemy for API documentation. -->
