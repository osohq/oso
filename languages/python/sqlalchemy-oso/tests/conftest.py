import pytest
from oso import Oso
from sqlalchemy import create_engine
from sqlalchemy.orm.session import Session

from sqlalchemy_oso.auth import register_models
from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3

from .models import ModelBase, Post, User

if USING_SQLAlchemy_v1_3:
    collect_ignore = ["test_advanced_queries_14.py"]


def print_query(query):
    print(query.statement.compile(), query.statement.compile().params)


@pytest.fixture
def post_fixtures():
    def create(session: Session):
        foo = User(id=0, username="foo")
        admin_user = User(id=1, username="admin_user", is_moderator=True)
        bad_user = User(id=2, username="bad_user", is_banned=True)
        users = [foo, admin_user, bad_user]

        posts = [
            Post(
                id=0, contents="foo public post", access_level="public", created_by=foo
            ),
            Post(
                id=1,
                contents="foo public post 2",
                access_level="public",
                created_by=foo,
            ),
            Post(
                id=3,
                contents="foo private post",
                access_level="private",
                created_by=foo,
            ),
            Post(
                id=4,
                contents="foo private post 2",
                access_level="private",
                created_by=foo,
            ),
            Post(
                id=5,
                contents="private for moderation",
                access_level="private",
                needs_moderation=True,
                created_by=foo,
            ),
            Post(
                id=6,
                contents="public for moderation",
                access_level="public",
                needs_moderation=True,
                created_by=foo,
            ),
            Post(
                id=7,
                contents="admin post",
                access_level="public",
                needs_moderation=True,
                created_by=admin_user,
            ),
            Post(
                id=8,
                contents="admin post",
                access_level="private",
                needs_moderation=True,
                created_by=admin_user,
            ),
            Post(
                id=9, contents="banned post", access_level="public", created_by=bad_user
            ),
        ]

        for p in posts:
            session.add(p)

        for u in users:
            session.add(u)

    return create


@pytest.fixture
def fixture_data(session, post_fixtures):
    post_fixtures(session)
    session.commit()


@pytest.fixture
def db_uri():
    return "sqlite:///:memory:"


@pytest.fixture
def engine(db_uri):
    try:  # SQLAlchemy 1.4
        engine = create_engine(db_uri, enable_from_linting=False)
    except TypeError:  # SQLAlchemy 1.3
        engine = create_engine(db_uri)
    ModelBase.metadata.create_all(engine)
    return engine


@pytest.fixture
def session(engine):
    return Session(bind=engine)


@pytest.fixture
def oso():
    oso = Oso()
    register_models(oso, ModelBase)
    return oso
