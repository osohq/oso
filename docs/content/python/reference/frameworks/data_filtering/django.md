---
docname: data-filtering/django
images: {}
title: Filter collections with Django
description: Learn to use Oso's Django integration to filter collections of data.
---

# Filter collections with Django

The `django-oso` library can enforce policies over Django models. This allows
policies to control access to collections of objects without needing to
authorize each object individually.

## Installation

The Oso Django integration is available on [PyPI](https://pypi.org/project/django-oso/) and can be installed using
`pip`:

```console
$ pip install django-oso
```

## Usage

The easiest way to prepare your existing Django models for use in a list
filtering policy is to have them inherit from
`django_oso.models.AuthorizedModel()`, a thin wrapper around
`django.models.Model` that calls `django_oso.auth.authorize_model()`
under the hood to return Django `QuerySet`s with authorization filters applied.

The policies you write will largely look the same with or without the list
filtering feature, and the Oso engine will follow similar evaluation paths.

In the list filtering case, Oso consults the policy to build up a list of
conditions that must be met in order for a model to be authorized. These
conditions are translated into Django ORM filters and applied to the query
before retrieving objects from the database.

## Example

Let’s look at an example usage of this library. Our example is a social media
app that allows users to view posts. There is a `User` model and a `Post`
model:

{{< literalInclude path="examples/data_access/django/example/app/models.py" >}}

We want to enforce the following authorization scheme for posts:

1. Anyone is allowed to `GET` any public post.
2. A user is allowed to `GET` their own private posts.
3. A user is allowed to `GET` private posts made by users they manage (defined
   through the `user.manager` relationship).

The corresponding policy looks as follows:

{{< literalInclude path="examples/data_access/django/example/app/policy/example.polar" >}}

### Trying it out

If you want to follow along, clone the Oso repository from [GitHub](https://github.com/osohq/oso) and `cd`
into it and then into the `docs/examples/data_access/django` directory.
Then, run `make setup` to install dependencies (primarily Django and
`django-oso`) and seed the database.

The database now contains a set of four posts made by two users:

```py
manager = User(username="manager")
user = User(username="user", manager=manager)

Post(contents="public user post", access_level="public", creator=user)
Post(contents="private user post", access_level="private", creator=user)
Post(contents="public manager post", access_level="public", creator=manager)
Post(contents="private manager post", access_level="private", creator=manager)
```

Once everything is set up, run `python example/manage.py runserver` to start
the Django app. We can now use cURL to interact with the application.

A guest user may view public posts:

```console
$ curl localhost:8000/posts
1 - @user - public - public user post
3 - @manager - public - public manager post
```

A non-manager may view public posts and their own private posts:

```console
$ curl --user user:user localhost:8000/posts
1 - @user - public - public user post
2 - @user - private - private user post
3 - @manager - public - public manager post
```

A manager may view public posts, their own private posts, and private posts of
their direct reports:

```console
$ curl --user manager:manager localhost:8000/posts
1 - @user - public - public user post
2 - @user - private - private user post
3 - @manager - public - public manager post
4 - @manager - private - private manager post
```

## How it works

`QuerySet`s containing authorized models are automatically filtered using
constraints derived from the policy.

For example, the above policy has the following rule:

{{< literalInclude path="examples/data_access/django/example/app/policy/example.polar"
                   from="their own private posts"
                   to="created by users who they manage" >}}

When determining which `Post` objects `User(id=2)` is authorized to see,
the `django-oso` adapter converts the constraints on Post expressed in this
rule into a Django `Q` filter:

```py
(AND: ('access_level', 'private'), ('creator__pk', 2))
```

When composed with filters generated from the other rules, the `QuerySet` is
scoped down to include only authorized objects. The result is the following SQL
statement, with the highlighted clause corresponding to the above filter:

```sql
SELECT "app_post"."id", "app_post"."contents", "app_post"."access_level", "app_post"."creator_id"
FROM "app_post"
WHERE "app_post"."id" IN (
  SELECT DISTINCT U0."id"
  FROM "app_post" U0
  WHERE (
    U0."access_level" = 'public' OR
    (U0."access_level" = 'private' AND U0."creator_id" = 2)));
```

## Limitations

There are some operators and features that do not currently work with the
Django adapter when used **anywhere in the policy**:

* The `cut` operator.
* Rules that rely on ordered execution based on class inheritance.

Some operations cannot be performed on **authorized models** in rules used with
the Django adapter. These operations can still be used on regular Django models
or Python objects:

* Application method calls.
* Arithmetic operators.
