---
title: "Filter Data (Oso 0.20.0 Beta)"
weight: 1
showContentForAnyLanguage: true
---

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Data Filtering Preview

{{% ifLangExists %}}

{{< ifLang not="node" >}}
{{< ifLang not="python" >}}
{{< ifLang not="ruby" >}}
Data filtering is coming soon for {{< lang >}}!

If you want to get data filtering in your app now or just want to
register your interest for Data Filtering in {{< lang >}}, [drop into our Slack](https://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Data%20filtering%20support%20for%20{{< currentLanguage >}}&body=I%27m%20interested%20in%20data%20filtering%20support%20for%20{{< currentLanguage >}}">send an email</a>
to our engineering team and we'll unblock you.
{{< /ifLang >}}
{{< /ifLang >}}
{{< /ifLang >}}

{{< ifLang not="rust" >}}
{{< ifLang not="go" >}}
{{< ifLang not="java" >}}

## What is data filtering
When you evaluate an Oso policy (using `is_allowed`) for a specific `actor`, `action` and `resource`, Oso evaluates the allow rule(s) you have defined to determine if that `actor` is allowed to do that `action` on that `resource`. For instance if you have a policy like this.


```polar
allow(actor, "get", doc: Document) if doc.owner = actor;
```

Oso checks that the passed in `doc`'s owner field is equal to the passed in actor. For example if the actor is `"steve"` and the document is `{owner: "steve", content: "..."}`
then `"steve"` would be allowed to `"get"` that document.

Data filtering is asking a slightly different question of the policy. Instead of asking "Can this actor do this action on this specific resource?", we want to ask
"What are all the resources that this actor can do this specific action on?".  One way to answer this question would be to take every Document in the system and call
`is_allowed` on it. This isn't efficient and many times is just impossible. There could be thousands of Documents in a database but only 3 that have the owner `"steve"`.
Instead of fetching every document and passing it into Oso, we would like to ask our database for only the documents that have the owner `"steve"`. This process of
filtering the data in our data store, based on the logic in our policy is what we call "Data Filtering".

{{% callout "ORM Integrations" "blue" %}}

If you are using one of our ORM integration libraries like `sqlalchemy-oso` or `django-oso` data filtering is already built in and you won't have to worry about integrating
it yourself. See docs for the ORM library instead.

{{% /callout %}}

## How data filtering works
Data filtering works by evaluating the policy without passing in a resource. Instead of checking things on the resource
like `resource.name = "steve"`, we collect up what each of those individual checks would be. These are called `Constraint`s. We
then call a function the user has registered with those constraints so that the user can query their own data store with them.

## How to use data filtering
To use data filtering you need to provide two additional things when you register your classes.

### Types
Oso needs to know the types of all the fields in your class. This is how we know what `resource.name` will be when we don't have
a concrete resource to check the field on. This lets Polar code work the same way it does normally and things like specializers match.
There is one special type called `Relationship` that tells Polar the field refers to a related object. This lets you reference a
related object in Polar and tells us how the current object is related to the other object.

### Fetchers
The other thing Oso has to know to use data filtering are how to fetch data. These are functions that take as input a list of `Constraint`
objects. The function is then responsible for selecting data that matches all of the constraint from the database, or an API, or wherever
the data lives. This is the place that data filtering integrates with your data store.

### Using
You can pass both of these things as arguments when registering a class and then you can use data filtering. Here's  an example.

## Example

{{< ifLang "python" >}}

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

{{< ifLang "ruby" >}}

Here's a small ActiveRecord example, but the principles should apply to any ORM.

```ruby
require 'active_record'
require 'sqlite3'

# define models
class User < ActiveRecord::Base
  include ActiveRecordFetcher # see below
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
end

class Repo < ActiveRecord::Base
  include ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :issues, foreign_key: :repo_name
end

class Org < ActiveRecord::Base
  include ActiveRecordFetcher
  self.primary_key = :name
  has_many :users, foreign_key: :org_name
  has_many :repos, foreign_key: :org_name
end

class Issue < ActiveRecord::Base
  include ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :repo, foreign_key: :repo_name
end

# create database
DB_FILE = 'test.db'

db = SQLite3::Database.new DB_FILE

db.execute <<-SQL
  create table orgs (
    name varchar(16) not null primary key
  );
SQL

db.execute <<-SQL
  create table users (
    name varchar(16) not null primary key,
    org_name varchar(16) not null
  );
SQL

db.execute <<-SQL
  create table repos (
    name varchar(16) not null primary key,
    org_name varchar(16) not null
  );
SQL

db.execute <<-SQL
  create table issues (
    name varchar(16) not null primary key,
    repo_name varchar(16) not null
  );
SQL

ActiveRecord::Base.establish_connection(
  adapter: 'sqlite3',
  database: DB_FILE
)


# create some records!

apple = Org.create name: 'apple'
osohq = Org.create name: 'osohq'

ios = Repo.create name: 'ios', org: apple
oso_repo = Repo.create name: 'oso', org: osohq
demo_repo = Repo.create name: 'demo', org: osohq

leina = User.create name: 'leina', org: osohq
steve = User.create name: 'steve', org: apple

bug = Issue.create name: 'bug', repo: oso_repo
laggy = Issue.create name: 'laggy', repo: ios
```

{{% /ifLang %}}

{{< ifLang "node" >}}

Here's a small TypeORM example, but the principles should apply to any ORM.

```js
import 'reflect-metadata';
import { Entity, PrimaryColumn, Column, createConnection } from 'typeorm';

@Entity()
export class Org {
  @PrimaryColumn()
  id!: string;
}

@Entity()
export class Repo {
  @PrimaryColumn()
  id!: string;

  @Column()
  orgId!: string;
}

@Entity()
export class User {
  @PrimaryColumn()
  id!: string;

  @Column()
  orgId!: string;
}

  const connection = await createConnection({
    type: 'sqlite',
    database: `:memory:`,
    entities: [Org, Repo, User],
    synchronize: true,
    logging: false,
  });

  let orgs = connection.getRepository(Org);
  let repos = connection.getRepository(Repo);
  let users = connection.getRepository(User);

  async function mkOrg(id: string) {
    let org = new Org()
    org.id = id;
    await orgs.save(org);
    return org;
  }

  async function mkRepo(id: string, orgId: string) {
    let repo = new Repo()
    repo.id = id;
    repo.orgId = orgId;
    await repos.save(repo);
    return repo;
  }

  async function mkUser(id: string, orgId: string) {
    let user = new User()
    user.id = id;
    user.orgId = orgId;
    await users.save(user);
    return user;
  }

  let apple = await mkOrg("apple")
  let osoOrg = await mkOrg("osohq")

  let ios = await mkRepo("ios", "apple")
  let osoRepo = await mkRepo("oso", "osohq")
  let demoRepo = await mkRepo("demo", "osohq")

  let leina = await mkUser("leina", "osohq")
  let steve = await mkUser("steve", "apple")
```

{{% /ifLang %}}

For each class, we need to define a fetching function. This is a function that takes a list of constraints and returns all the instances that match them.
In some cases you might be filtering an in-memory array; in other cases, you might be constructing a request to an external service to fetch data. In this
case, we are turning the constraints into a database query.

{{% callout "Handling Constraints" "orange" %}}

It is very important that you handle every constraint that is passed in. Missing any will result in
returning data that the user is not allowed to see.

{{% /callout %}}

Each constraint is an object with a `kind`, `field` and `value`.
There are three kinds of constraints.

* `"Eq"` constraints mean that the field `field` must be equal to the value `value`
* `"In"` constraints mean that the value of the field `field` must be one of the values in the list `value`.
* `"Contains"` constraints only apply if the field `field` is a list. It means this list must contain all the values in `value`.
  If none of your fields are lists you wont get passed this one.

{{< ifLang "python" >}}

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

{{< ifLang "ruby" >}}

```ruby
  # A module to autogenerate a fetcher function for an ActiveRecord subclass.
  # `include` it in `Class`and the fetcher is accessible in `Class::FETCHER`
  module ActiveRecordFetcher
    def self.included(base)
      base.class_eval do
        const_set(:FETCHER, lambda do |cons|
          cons.reduce(self) do |q, con|
            raise "Unsupported constraint kind: #{con.kind}" unless %w[Eq In].include? con.kind

            q.where(
              if con.field.nil?
                { primary_key => con.value.send(primary_key) }
              else
                { con.field => con.value }
              end
            )
          end
        end)
      end
    end
  end
```
{{% /ifLang %}}

{{< ifLang "node" >}}

```js
  function fromRepo(repo: any, name: string, constraints: any) {
    let query = repo.createQueryBuilder(name);
    for (let i in constraints) {
      let c = constraints[i];
      let clause;
      switch (c.kind) {
        case 'Eq':
          {
            clause = `${name}.${c.field} = :${c.field}`;
          }
          break;
        case 'In':
          {
            clause = `${name}.${c.field} IN (:...${c.field})`;
          }
          break;
      }
      let param: any = {};
      param[c.field] = c.value;
      query.andWhere(clause, param);
    }
    return query.getMany();
  }

  function getOrgs(constraints: any) {
    return fromRepo(orgs, 'org', constraints);
  }

  function getRepos(constraints: any) {
    return fromRepo(repos, 'repo', constraints);
  }
```

{{% /ifLang %}}

When you register classes you need to specify two new things. One is `types` which is a map that says what the type of each
field in the class is. This can be base types like `String` or another registered class or it can be a `Relationship`.

The Relationship specifies the type of the related value. It also says how it's related. Any instance of the other type that
has their `other_field` equal to the current objects `my_field` is related. `kind` can be either `"parent"` which means there
is one related instance, or `"children"` which means there are many. Internally the fields are used to create constraints and
the related instances are fetched using the fetching functions.

The other new thing to specify when registering a class is the fetching function for that class.

{{< ifLang "python" >}}

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

{{< ifLang "ruby" >}}
```ruby
require 'oso'

oso = Oso.new
Relationship = Oso::Polar::DataFiltering::Relationship

oso.register_class(
  User,
  fetcher: User::FETCHER,
  fields: {
    name: String,
    org_name: String,
    org: Relationship.new(
      kind: 'parent',
      other_type: 'Org',
      my_field: 'org_name',
      other_field: 'name'
    )
  }
)

oso.register_class(
  Org,
  fetcher: Org::FETCHER,
  fields: {
    name: String,
    users: Relationship.new(
      kind: 'children',
      other_type: 'User',
      my_field: 'name',
      other_field: 'org_name'
    ),
    repos: Relationship.new(
      kind: 'children',
      other_type: 'Repo',
      my_field: 'name',
      other_field: 'org_name'
    )
  }
)

oso.register_class(
  Repo,
  fetcher: Repo::FETCHER,
  fields: {
    name: String,
    org_name: String,
    org: Relationship.new(
      kind: 'parent',
      other_type: 'Org',
      my_field: 'org_name',
      other_field: 'name'
    )
  }
)

oso.register_class(
  Issue,
  fetcher: Issue::FETCHER,
  fields: {
    name: String,
    repo_name: String,
    repo: Relationship.new(
      kind: 'parent',
      other_type: 'Repo',
      my_field: 'repo_name',
      other_field: 'name'
    )
  }
)
```
{{% /ifLang %}}

{{< ifLang "node" >}}

```js
  import { Oso, Relationship } from 'oso';

  const oso = new Oso();

  const orgType = new Map();
  orgType.set('id', String);
  oso.registerClass(Org, 'Org', orgType, getOrgs);

  const repoType = new Map();
  repoType.set('id', String);
  repoType.set('orgId', String);
  repoType.set('org', new Relationship('parent', 'Org', 'orgId', 'id'));
  oso.registerClass(Repo, 'Repo', repoType, getRepos);

  const userType = new Map();
  userType.set('id', String);
  userType.set('orgId', String);
  oso.registerClass(User, 'User', userType);
```

{{% /ifLang %}}

One everything is set up we can use the new "get allowed resources" method to filter a Class to all the instances that the user is allowed to preform the action on.

{{< ifLang "python" >}}
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

{{< ifLang "ruby" >}}

```ruby
oso.load_str <<~POL
  allow(user: User, "read", repo: Repo) if
    user.org = repo.org;
  allow(user: User, "edit", issue: Issue) if
    user.org = issue.repo.org;
POL

leina_repos = oso.get_allowed_resources leina, 'read', Repo
raise unless leina_repos == [oso_repo, demo_repo]

steve_issues = oso.get_allowed_resources steve, 'edit', Issue
raise unless steve_issues == [laggy]

```

{{% /ifLang %}}

{{< ifLang "node" >}}

```js
  oso.loadStr(`
    allow(user: User, "read", repo: Repo) if
      org = repo.org and
      user.orgId = org.id;
  `);
  let leinaRepos = await oso.getAllowedResources(leina, "read", Repo)
  console.log(leinaRepos)
  assert(leinaRepos == [osoRepo, demoRepo])
```

{{% /ifLang %}}

## Limitations
Currently there are some limitations to what you can do while using data filtering. You can not call any methods on the passed in resource (or any of its properties). You also can not pass the resource as an argument to a method. Many cases where you would want to do this are better handled by Relationship fields.

Some Polar expressions are not supported but may be in the future. `not`, `cut` and `forall` are not allowed in policies that want to use data filtering. Numeric comparisons with the `< > <= >= !=` operators are also not yet supported.

For now, Relationships only support matching on a single field.

{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLangExists %}}
