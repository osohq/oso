# docs: begin-a1
# We're using sqlalchemy here, but you can use data filtering with any ORM
from sqlalchemy import create_engine
from sqlalchemy.types import String, Boolean, Integer
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker, relationship
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()


class Repository(Base):
    __tablename__ = "repos"

    id = Column(String(), primary_key=True)


class User(Base):
    __tablename__ = "users"

    id = Column(String(), primary_key=True)


class RepoRole(Base):
    __tablename__ = "repo_roles"
    id = Column(Integer, primary_key=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False)
    repo_id = Column(String, ForeignKey("repos.id"), nullable=False)
    user = relationship("User", backref="repo_roles", lazy=True)
    name = Column(String, index=True)


engine = create_engine("sqlite:///:memory:")

Session = sessionmaker(bind=engine)
session = Session()

Base.metadata.create_all(engine)

# Here's some data to work with ...
ios = Repository(id="ios")
oso_repo = Repository(id="oso")
demo_repo = Repository(id="demo")

leina = User(id="leina")
steve = User(id="steve")

role_1 = RepoRole(user_id="leina", repo_id="oso", name="contributor")
role_2 = RepoRole(user_id="leina", repo_id="demo", name="maintainer")

objs = {
    "leina": leina,
    "steve": steve,
    "ios": ios,
    "oso_repo": oso_repo,
    "demo_repo": demo_repo,
    "role_1": role_1,
    "role_2": role_2,
}
for obj in objs.values():
    session.add(obj)
session.commit()
# docs: end-a1

# docs: begin-a2
# build_query takes a list of filters and returns a query
def build_query(filters):
    query = session.query(Repository)
    for filter in filters:
        assert filter.kind in ["Eq", "In"]
        field = getattr(Repository, filter.field)
        if filter.kind == "Eq":
            query = query.filter(field == filter.value)
        elif filter.kind == "In":
            query = query.filter(field.in_(filter.value))
    return query


# exec_query takes a query and returns a list of resources
def exec_query(query):
    return query.all()


# combine_query takes two queries and returns a new query
def combine_query(q1, q2):
    return q1.union(q2)


from oso import Oso

oso = Oso()

oso.register_class(
    Repository,
    types={
        "id": str,
    },
    build_query=build_query,
    exec_query=exec_query,
    combine_query=combine_query,
)

oso.register_class(User, types={"id": str, "repo_roles": list})
# docs: end-a2

with open("../policy_a.polar") as f:
    policy_a = f.read()

# docs: begin-a3
oso.load_str(policy_a)
# Verify that the policy works as expected
leina_repos = list(oso.authorized_resources(leina, "read", Repository))
assert leina_repos == [demo_repo, oso_repo]
# docs: end-a3
