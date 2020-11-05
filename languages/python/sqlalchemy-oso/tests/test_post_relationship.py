"""Standardized tests for adapters based on the Post model.

Tests come from the relationship document & operations laid out there
https://www.notion.so/osohq/Relationships-621b884edbc6423f93d29e6066e58d16.
"""
import pytest

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.orm.session import Session
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy import Column, Integer, String, Enum, Boolean

from oso import Oso
from sqlalchemy_oso.auth import authorize_model, register_models

ModelBase = declarative_base(name='ModelBase')

def print_query(query):
    print(query.statement.compile(), query.statement.compile().params)

class Post(ModelBase):
    __tablename__ = 'posts'

    id = Column(Integer, primary_key=True)
    contents = Column(String)
    access_level = Column(Enum('public', 'private'), nullable=False)

    needs_moderation = Column(Boolean, nullable=False, default=False)

@pytest.fixture
def post_fixtures():
    def create(session: Session):
        posts = [
            Post(id=0, contents="public post", access_level='public'),
            Post(id=1, contents="public post 2", access_level='public'),

            Post(id=3, contents="private post", access_level='private'),
            Post(id=4, contents="private post 2", access_level='private'),

            Post(id=5, contents="private for moderation", access_level='private',
                 needs_moderation=True),
            Post(id=6, contents="public for moderation", access_level='public',
                 needs_moderation=True)
        ]

        for p in posts:
            session.add(p)

    return create

@pytest.fixture
def fixture_data(post_fixtures):
    return post_fixtures

@pytest.fixture
def engine(fixture_data):
    engine = create_engine('sqlite:///:memory:')
    ModelBase.metadata.create_all(engine)

    session = Session(bind=engine)
    fixture_data(session)
    session.commit()

    return engine

@pytest.fixture
def session(engine):
    return Session(bind=engine)

@pytest.fixture
def oso():
    oso = Oso()
    register_models(oso, ModelBase)
    return oso

def test_authorize_model_basic(session, oso):
    """Test that a simple policy with checks on non-relationship attributes is correct."""
    oso.load_str('allow("user", "read", post: Post) if post.access_level = "public";')
    oso.load_str('allow("user", "write", post: Post) if post.access_level = "private";')
    oso.load_str('allow("admin", "read", post: Post);')
    oso.load_str('allow("moderator", "read", post: Post) if '
                 '(post.access_level = "private" or post.access_level = "public") and '
                 'post.needs_moderation = true;')

    posts = authorize_model(oso, "user", "read", session, Post)

    assert posts.count() == 3
    assert posts.all()[0].contents == "public post"
    assert posts.all()[0].id == 0

    posts = authorize_model(oso, "user", "write", session, Post)

    assert posts.count() == 3
    assert posts.all()[0].contents == "private post"
    assert posts.all()[1].contents == "private post 2"

    posts = authorize_model(oso, "admin", "read", session, Post)
    assert posts.count() == 6

    posts = authorize_model(oso, "moderator", "read", session, Post)
    print_query(posts)
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    posts = authorize_model(oso, "guest", "read", session, Post)
    assert posts.count() == 0
