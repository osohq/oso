from .routes import serialize, app
from . import models

from oso import Oso

from sqlalchemy import Column, String, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy import create_engine

# please forgive this nightmare -dave


oso = Oso()

engine = create_engine('sqlite://')
Session = sessionmaker(bind=engine)

Base = declarative_base(bind=engine)

class Repository(Base):
    __tablename__ = 'repo'

    name = Column(String(128), primary_key=True)
    is_public = Column(Boolean)

Base.metadata.create_all()

# docs: begin-data-filtering
# This is an example implementation for the SQLAlchemy ORM, but you can
# use any ORM with this API.
def get_repositories(constraints):
    query = Session().query(Repository)
    for constraint in constraints:
        value = constraint.value
        # If the field is None, this constraint is comparing against
        # a repository object.
        if constraint.field is None:
            value = value.name
            field = Repository.name
        else:
            field = getattr(Repository, constraint.field)

        if constraint.kind == "Eq":
            query = query.filter(field == value)
        elif constraint.kind == "In":
            query = query.filter(field.in_(value))
        else:
            raise NotImplementedError("unsupported constraint type")

    return query

oso.register_class(models.User)
oso.register_class(
    Repository,
    types={
		# Tell Oso the types of fields you will use in your policy.
		"is_public": bool
	},
    build_query=get_repositories,
    exec_query=lambda q: q.all(),
    combine_query=lambda q1, q2: q1.union(q2),
)

oso.load_files(["main.polar"])
# docs: end-data-filtering

class User:
    @staticmethod
    def get_current_user():
        return models.User(roles=[{"name": "admin", "repository": Repository(name="gmail")}])

# docs: begin-list-route
@app.route("/repos")
def repo_list():
    repositories = oso.authorized_resources(
        User.get_current_user(),
        "read",
        Repository)

    return serialize(repositories)
# docs: end-list-route
