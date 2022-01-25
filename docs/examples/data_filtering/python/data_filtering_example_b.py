# docs: begin-b1
from sqlalchemy import create_engine, not_, or_, and_, false
from sqlalchemy.types import String, Boolean, Integer
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker, relationship
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()


class Organization(Base):
    __tablename__ = "orgs"

    id = Column(String(), primary_key=True)


# Repositories belong to Organizations
class Repository(Base):
    __tablename__ = "repos"

    id = Column(String(), primary_key=True)
    org_id = Column(String, ForeignKey("orgs.id"), nullable=False)


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


class OrgRole(Base):
    __tablename__ = "org_roles"
    id = Column(Integer, primary_key=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False)
    org_id = Column(String, ForeignKey("orgs.id"), nullable=False)
    user = relationship("User", backref="org_roles", lazy=True)
    name = Column(String, index=True)


engine = create_engine("sqlite:///:memory:")

Session = sessionmaker(bind=engine)
session = Session()

Base.metadata.create_all(engine)

# Here's some more test data
osohq = Organization(id="osohq")
apple = Organization(id="apple")

ios = Repository(id="ios", org_id="apple")
oso_repo = Repository(id="oso", org_id="osohq")
demo_repo = Repository(id="demo", org_id="osohq")

leina = User(id="leina")
steve = User(id="steve")

role_1 = OrgRole(user_id="leina", org_id="osohq", name="owner")

objs = {
    "leina": leina,
    "steve": steve,
    "osohq": osohq,
    "apple": apple,
    "ios": ios,
    "oso_repo": oso_repo,
    "demo_repo": demo_repo,
    "role_1": role_1,
}
for obj in objs.values():
    session.add(obj)
session.commit()
# docs: end-b1

# docs: begin-b2
from oso import Oso, Relation
from polar.data.adapter.sqlalchemy_adapter import SqlAlchemyAdapter

oso = Oso()

oso.set_data_filtering_adapter(SqlAlchemyAdapter(session))

oso.register_class(
    Organization,
    fields={
        "id": str,
    },
)

oso.register_class(
    Repository,
    fields={
        "id": str,
        # Here we use a Relation to represent the logical connection between an Organization and a Repository.
        # Note that this only goes in one direction: to access repositories from an organization, we'd have to
        # add a "many" relation on the Organization class.
        "organization": Relation(
            kind="one", other_type="Organization", my_field="org_id", other_field="id"
        ),
    },
)

oso.register_class(User, fields={"id": str, "repo_roles": list})
# docs: end-b2

with open("policy_b.polar") as f:
    policy_a = f.read()

# docs: begin-b3
oso.load_str(policy_a)
leina_repos = list(oso.authorized_resources(leina, "read", Repository))
assert leina_repos == [oso_repo, demo_repo]
# docs: end-b3
