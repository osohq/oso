---
title: "Data Filtering Preview"
weight: 1
showContentForAnyLanguage: true
---

# Data Filtering Preview

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

{{< ifLangExists >}}
{{% ifLang not="node" %}}
{{% ifLang not="python" %}}
{{% ifLang not="ruby" %}}

Data filtering is coming soon for {{< lang >}}!

If you want to get data filtering in your app now or just want to
register your interest for Data Filtering in {{< lang >}} [drop into our Slack](http://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Data%20filtering%20support%20for%20{{< currentLanguage >}}&body=I%27m%20interested%20in%20data%20filtering%20support%20for%20{{< currentLanguage >}}">send an email</a>.
to our engineering team and we'll unblock you.
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLangExists %}}

## What is data filtering
When you evaluate an oso policy (using `is_allowed`) for a specific `actor`, `action` and `resource`, oso evaluates the allow rule(s) you have defined to determine if that `actor` is allowed to do that `action` on that `resource`. For instance if you have a policy like this.


```polar
allow(actor, "get", doc: Document) if doc.owner = actor;
```

Oso checks that the passed in `doc`'s owner field is equal to the passed in actor. Eg. if the actor is `"steve"` and the document is `{owner: "steve", content: "..."}` Then `"steve"` would be allowed to `"get"` that document.

Data filtering is asking a slightly different question of the policy. Instead of asking "Can this actor do this action on this specific resource?", we want to ask "What are all the resources that this actor can do this specific action on?".
One way to answer this question would be to take every Document in the system and call `is_allowed` on it. This isn't efficient and many times is just impossible. There could be thousands of Documents in a database but only 3 that have the owner "steve". Instead of fetching every document and passing it into oso, we would like to ask our database for only the documents that have the owner "steve". This process of filtering the data in our data store, based on the logic in our policy is what we call "Data Filtering".

{{% callout "ORM Integrations" "blue" %}}

If you are using one of our ORM integration libraries like sqlalchemy-oso or django-oso data filtering is already built in and you won't have to worry about integrating it yourself. See docs for the orm library instead.

{{% /callout %}}

## How data filtering works
Data filtering works by evaluating the policy without passing in a resource. Instead of checking things on the resource
like `resource.name = "steve"`, we collect up what each of those individual checks would be. These are called `Constraint`s. We then call a function the user has registered with those constraints so that the user can query their own data store with them.

## How to use data filtering
To use data filtering you need to provide two additional things when you register your classes.

### Types
Oso needs to know the types of all the fields in your class. This is how we know what `resource.name` will be when we don't have a concrete resource to check the field on. This lets polar code work the same way it does normally and things like specializers match.
There is one special type called `Relationship` that tells polar the field refers to a related object. This lets you reference a related object in polar and tells us how the current object is related to the other object.

### Fetchers
The other thing oso has to know to use data filtering are how to fetch data. These are functions that take as input a list of `Constraint` objects. The function is then responsible for selecting data that matches all of the constraint from the database, or an api, or wheverver the data lives. This is the place that data filtering integrates with your data store. 

### Using
You can pass both of these things as arguments when registering a class and then you can use data filtering. Here's  an example.

## Example

{{% ifLang "python" %}}

Here's a small SQLAlchemy example, but the principles should apply to any ORM.

```python
from sqlalchemy import create_engine
from sqlalchemy.types import String, Boolean
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()


class Org(Base):
    __tablename__ = "orgs"

    id = Column(String(), primary_key=True)


class Repo(Base):
    __tablename__ = "repos"

    id = Column(String(), primary_key=True)
    org_id = Column(String, ForeignKey("orgs.id"))


class User(Base):
    __tablename__ = "users"

    id = Column(String(), primary_key=True)
    org_id = Column(String, ForeignKey("orgs.id"))


engine = create_engine("sqlite:///:memory:")

Session = sessionmaker(bind=engine)
session = Session()

Base.metadata.create_all(engine)

apple = Org(id="apple")
osohq = Org(id="osohq")

ios = Repo(id="ios", org_id="apple")
oso_repo = Repo(id="oso", org_id="osohq")
demo_repo = Repo(id="demo", org_id="osohq")

leina = User(id="leina", org_id="osohq")
steve = User(id="steve", org_id="apple")

objs = {
    "leina": leina,
    "steve": steve,
    "apple": apple,
    "osohq": osohq,
    "ios": ios,
    "oso_repo": oso_repo,
    "demo_repo": demo_repo,
}
for obj in objs.values():
    session.add(obj)
session.commit()
```

{{% /ifLang %}}

For each class, we need to define a fetching function. This is a function that takes a list of constraints and returns all the instances that match them. In some cases you might be filtering an in memory array, in other cases you might be constructing a request to an external service to fetch data. In this case we are turning the constraints into a database query.

{{% callout "Handling Constraints" "orange" %}}

It is very important that you handle every constraint that is passed in. Missing any will result in
returning data that the user is not allowed to see.

{{% /callout %}}

The constraints are object with a `kind`, `field` and `value`.
There are three kinds of constraints.

* `"Eq"` constraints mean that the field `field` must be equal to the value `value`
* `"In"` constraints mean that the value of the field `field` must be one of the values in the list `value`.
* `"Contains"` constraints only apply if the field `field` is a list. It means this list must contain all the values in `value`. If none of your fields are lists you wont get passed this one.

{{% ifLang "python" %}}

```python
def query_model(model, constraints):
    query = session.query(model)
    for constraint in constraints:
        assert constraint.kind in ["Eq", "In"]
        field = getattr(model, constraint.field)
        if constraint.kind == "Eq":
            query = query.filter(field == constraint.value)
        elif constraint.kind == "In":
            query = query.filter(field.in_(constraint.value))
    return query.all()


def get_orgs(constraints):
    return query_model(Org, constraints)


def get_repos(constraints):
    return query_model(Repo, constraints)
```

{{% /ifLang %}}

When you register classes you need to specify two new things. One is `types` which is a map that says what the type of each field in the class is. This can be base types like `String` or another registered class or it can be a `Relationship`.

The Relationship specifies the type of the related value. It also says how it's related. Any instance of the other type that has their `other_field` equal to the current objects `my_field` is related. `kind` can be either `"parent"` which means there is one related instance, or `"children"` which means there are many.
Internally the fields are used to create constraints and the related instances are fetched using the fetching functions.

The other new thing to specify when registering a class is the fetching function for that class.

{{% ifLang "python" %}}

```python
from polar import Relationship
from oso import Oso

oso = Oso()

oso.register_class(Org, types={"id": str}, fetcher=get_orgs)

oso.register_class(
    Repo,
    types={
        "id": str,
        "org_id": str,
        "org": Relationship(
            kind="parent", other_type="Org", my_field="org_id", other_field="id"
        ),
    },
    fetcher=get_repos,
)

oso.register_class(User, types={"id": str, "org_id": str})

```

{{% /ifLang %}}

One everything is set up we can use the new "get allowed resources" method to filter a Class to all the instances that the user is allowed to preform the action on.

{{% ifLang "python" %}}
```python
policy = """
allow(user: User, "read", repo: Repo) if
    org = repo.org and
    user.org_id = org.id;
"""
oso.load_str(policy)
leina_repos = list(oso.get_allowed_resources(leina, "read", Repo))
assert leina_repos == [oso_repo, demo_repo]
```

{{% /ifLang %}}

{{% ifLang "ruby" %}}

Ruby example coming soon.

{{% /ifLang %}}

{{% ifLang "node" %}}

JavaScript example coming soon.

{{% /ifLang %}}

## Limitations
Currently there are some limitations to what you can do while using data filtering. You can not call any methods on the passed in resource (or any of it's properties). You also can not pass the resource as an argument to a method. Many cases where you would want to do this are better handled by Relationship fields.

Some polar expressions are not supported but may be in the future. `not`, `cut` and `forall` are not allowed in policies that want to use data filtering. Numeric comparisons are also not yet supported. `< > <= >= !=`

For now, Relationships only support matching on a single field. 