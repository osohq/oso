from .routes import serialize, app
from . import models

from oso import Oso

from sqlalchemy import Column, String, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy import create_engine


# This example uses a separate Oso instance so that I can re-register classes
# with data filtering query builders but use the same `main.polar` policy.

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
def get_repositories(filters):
    query = Session().query(Repository)
    for filter in filters:
        value = filter.value

        if filter.field is None:
            # If the field is None, this filter is comparing against
            # the repository object, so we construct a query that makes sure
            # the primary key (name) matches.
            value = value.name
            field = Repository.name
        else:
            # Otherwise, we get the field to compare against.
            field = getattr(Repository, filter.field)

        # Build SQLAlchemy query based on filters.
        if filter.kind == "Eq":
            query = query.filter(field == value)
        elif filter.kind == "In":
            query = query.filter(field.in_(value))
        else:
            # See full guide to handle other constraint types.
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
