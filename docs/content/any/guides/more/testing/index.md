---
title: Test Your Policy
weight: 10
description: |
  Learn to test your Oso policies.
---

# Test your policy

In Oso, your authorization logic is separated from the rest of your application in a policy. This makes it very easy to test authorization in isolation.

Oso works with your favorite testing framework. We'll demonstrate with[pytest](https://docs.pytest.org/en/6.2.x/).

## Policy tests

You can test your policy through the same API you call your policy with `oso.authorize()`.

This example has one kind of resource, a `Repository` (like GitHub or GitLab). Users have roles of `contributor`, `maintainer`, and `admin` on each repository. Someone who has `admin` permissions on one repository does not have `admin` permissions for a different repository.

Initialize your Oso instance in a test fixture.

```python
@pytest.fixture(scope="session")
def oso():
    oso_instance = Oso()
    oso_instance.register_class(User)
    oso_instance.register_class(Repository)
    oso_instance.load_files(["app/main.polar"])
    return oso_instance
```

Let's test who has the ability to delete repositories.

Contributors do not have permission to delete repositories:

```python
def test_contributors_cannot_delete_repos(oso):
    repo = Repository("oso")
    contributor = User([Role(name="contributors", repository=repo)])
    with pytest.raises(ForbiddenError):
        oso.authorize(contributor, "delete", repo)
```

Maintainers do not have permission to delete repositories:

```python
def test_maintainers_cannot_delete_repos(oso):
    repo = Repository("oso")
    maintainer = User([Role(name="maintainer", repository=repo)])
    with pytest.raises(ForbiddenError):
        oso.authorize(maintainer, "delete", repo)
```

But, admins do have permission to delete repositories:

```python
def test_admins_can_delete_repos(oso):
    repo = Repository("oso")
    user = User([Role(name="admin", repository=repo)])
    assert oso.authorize(user, "delete", repo) == None
```

## Unit Testing Your Policy

The examples above test your whole policy. If your policy is complicated, you may want to test small pieces of it in the same way you would unit test your app code.

Oso provides `oso.query_rule` to query individual rules of your policy. Rules may return many results, so `oso.query_rule` is a generator—you can use `oso.query_rule_once` to ensure the query returns exactly one result.

Our policy has a rule, `has_role`, to look up roles on a repository. Here's how to test that logic is correct:

```python
def test_admin_users_have_admin_roles_on_repos(oso):
    repo = Repository.get_by_name("gmail")
    user = User([Role(name="admin", repository=repo)])
    assert oso.query_rule_once("has_role", user, "admin", repo)
```

## Integration Testing

Policy and unit tests confirm that the policy is correct, but do not confirm that your application's authorization behaves correctly as a whole. To test your app's behavior, write integration tests that test your routes end-to-end. Like all integration tests, you'll need a mock dataset to test against. Most web frameworks have a test client that can make this easy. In this case, we're querying Flask's `get` method directly.

```python
@pytest.fixture
def client():
    with app.test_client() as client:
        yield client

def test_invalid_access_404s(client):
    User.current_user = User([Role(name="admin", repository=repos_db["gmail"])])
    response = client.get("/repo/oso")
    assert response.status_code == 404

def test_valid_access_200s(client):
    User.current_user = User([Role(name="admin", repository=repos_db["gmail"])])
    response = client.get("/repo/gmail")
    assert response.status_code == 200
```

## Best practices: what kind of tests should I write?

Policy tests, unit tests, and integration tests are all useful.

Policy tests help ensure that your policy behaves correctly for a set of inputs. Spend most of your testing effort here, dipping in to unit tests as needed.

- Test each `allow` rule you write. Test as many combinations of `actor`, `action` and `resource` as you need to cover your policy's logic.

Unit tests allow you to use Test-Driven Development (TDD) to write your policy and to isolate particularly tricky logic for testing. These should work alongside your policy tests to test your policy's correctness.

- TDD is a great way to write a policy—as you add rules, test them with `oso.query_rule_once()`.
- In a policy with roles, test your `has_role` rules' integration with your app to make sure they find the correct roles for all resources.
- Test that your `has_relation(subject, relname, object)` rules find the `subject` of each relation properly. You can do that by adding an Oso `Variable`—Oso's `query_rule` will return all possible values for that variable. For instance, if you're testing a "parent" relation, make sure that `oso.query_rule('has_relation', (Variable('subject'), "parent", object))` returns the correct `subject`s.

Integration testing confirms that policy *enforcement* is correct in your application. Because integration tests exercise parts of your app that aren't Oso, try to minimize the amount of integration tests you write. 

- If you're calling `authorize` in each route handler, you should have one authorization test for every route.
- If you're able to call `authorize` in your middleware, it may only be necessary to test a few routes to confirm that your middleware is correctly covering your endpoints.

## For extra coverage, use decision tables to test combinations of inputs

You'll often want to test every combination of a set of properties, like every combination of role and permission. For that case, we recommend writing and testing a [decision table](https://en.wikipedia.org/wiki/Decision_table) of allowed actions.

```python
def test_combinations_of_role_and_action(oso):
    repository = Repository("oso")

    combinations = [
        ("contributor", "read",   repository, True ),
        ("contributor", "push",   repository, False),
        ("contributor", "delete", repository, False),
        ("maintainer",  "read",   repository, True ),
        ("maintainer",  "push",   repository, True ),
        ("maintainer",  "delete", repository, False),
        ("admin",       "read",   repository, True ),
        ("admin",       "push",   repository, True ),
        ("admin",       "delete", repository, True ),
    ]

    errors = []
    for role, action, repo, expected in combinations:
        user = User([Role(name=role, repository=repo)])
        try:
            oso.authorize(user, action, repo)
            actual = True
        except AuthorizationError:
            actual = False

        if actual != expected:
            errors.append(role + ":" + action)

    assert errors == []
```

## What's Next

- Check out our [How-To Guides](https://docs.osohq.com/guides.html) for more on using Polar
policies for authorization.
- Check out the [Polar reference](https://docs.osohq.com/reference/polar.html) for more on the Polar language and syntax.
