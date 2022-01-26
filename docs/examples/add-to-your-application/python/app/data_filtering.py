from pathlib import Path

from .routes import serialize, app
from . import models

from oso import Oso
from polar.data.adapter.sqlalchemy_adapter import SqlAlchemyAdapter

from sqlalchemy import Column, String, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy import create_engine


# This example uses a separate Oso instance so that I can re-register classes
# with data filtering query builders but use the same `main.polar` policy.

oso = Oso()

engine = create_engine("sqlite://")
Session = sessionmaker(bind=engine)

Base = declarative_base(bind=engine)


class Repository(Base):
    __tablename__ = "repo"

    name = Column(String(128), primary_key=True)
    is_public = Column(Boolean)


Base.metadata.create_all()

# docs: begin-data-filtering
oso.register_class(models.User)
oso.register_class(
    Repository,
    fields={
        # Tell Oso the types of fields you will use in your policy.
        "is_public": bool
    },
)

oso.set_data_filtering_adapter(SqlAlchemyAdapter(Session()))

oso.load_files([Path(__file__).parent / "main.polar"])
# docs: end-data-filtering


class User:
    @staticmethod
    def get_current_user():
        return models.User(
            roles=[{"name": "admin", "repository": Repository(name="gmail")}]
        )


# docs: begin-list-route
@app.route("/repos")
def repo_list():
    repositories = oso.authorized_resources(User.get_current_user(), "read", Repository)

    return serialize(repositories)


# docs: end-list-route
