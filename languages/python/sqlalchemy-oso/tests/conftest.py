import pytest

from sqlalchemy import create_engine
from sqlalchemy.orm.session import Session

from oso import Oso
from sqlalchemy_oso.auth import register_models

from .models import ModelBase, Post, User


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
def engine():
    engine = create_engine("sqlite:///:memory:")
    ModelBase.metadata.create_all(engine)
    return engine


@pytest.fixture
def session(engine):
    return Session(bind=engine, enable_baked_queries=False)


@pytest.fixture
def oso():
    oso = Oso()
    register_models(oso, ModelBase)
    return oso
