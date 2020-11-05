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

from sqlalchemy import Column, Integer, String

from oso import Oso
from sqlalchemy_oso.auth import authorize_model, register_models

ModelBase = declarative_base(name='ModelBase')

class Post(ModelBase):
    __tablename__ = 'posts'

    id = Column(Integer, primary_key=True)
    contents = Column(String)

@pytest.fixture
def post_fixtures():
    def create(session: Session):
        posts = [
            Post(id=0, contents="Important post."),
            Post(id=1, contents="Not important post.")
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
    oso.load_str('allow("user", "read", post: Post) if post.contents = "Important post.";')

    posts = authorize_model(oso, "user", "read", session, Post)

    assert posts.count() == 1
    assert posts.all()[0].contents == "Important post."
    assert posts.all()[0].id == 0
