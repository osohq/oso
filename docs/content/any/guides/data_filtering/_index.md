---
title: "Data Filtering Preview"
weight: 1
showContentForAnyLanguage: true
---

# Data Filtering Preview

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

{{% callout "Early Preview" "orange" %}}

Data filtering is currently an Early Preview. If you have any trouble using it or it doesn't work with your policy [drop into our Slack](http://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Data%20filtering%20help%20for%20{{< currentLanguage >}}&body=I%20need%20data%20filtering%20help%20in%20{{< currentLanguage >}}">send us an email</a>.

Thanks for trying it out!

{{% /callout %}}

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

```python
from sqlalchemy import create_engine
from sqlalchemy.types import String, Boolean
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from polar import Relationship
from oso import Oso

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