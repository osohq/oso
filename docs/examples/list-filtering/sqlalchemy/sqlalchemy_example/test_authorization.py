from pathlib import Path

import pytest

from sqlalchemy import create_engine
from sqlalchemy.orm import Session

from oso import Oso
from sqlalchemy_oso.hooks import authorize_query
from sqlalchemy_oso.auth import register_models

from .models import Post, User, Model

POLICY_FILE = Path(__file__).absolute().parent / 'policy.polar'

@pytest.fixture
def engine():
    engine = create_engine('sqlite:///:memory:')
    Model.metadata.create_all(engine)
    return engine

@pytest.fixture
def session(engine):
    return Session(
        bind=engine,
        # Baked queries must be disabled to use oso.
        enable_baked_queries=False
    )

@pytest.fixture
def oso():
    return Oso()

@pytest.fixture
def policy(oso):
    register_models(oso, Model)
    oso.load_file(POLICY_FILE)

@pytest.fixture
def test_data(session):
    user = User(username='user')
    manager = User(username='manager', manages=[user])

    public_user_post = Post(contents='public_user_post',
                            access_level='public',
                            created_by=user)
    private_user_post = Post(contents='private_user_post',
                            access_level='private',
                            created_by=user)
    private_manager_post = Post(contents='private manager post',
                                access_level='private',
                                created_by=manager)
    public_manager_post = Post(contents='public manager post',
                               access_level='public',
                               created_by=manager)

    models = {name: value for name, value in locals().items() if isinstance(value, Model)}
    for instance in models.values():
        session.add(instance)

    session.commit()

    return models

def test_basic(oso, policy, session, test_data):
    posts = session.query(Post)

    authorized_posts = authorize_query(
        posts,
        lambda: oso,
        lambda: test_data['user'],
        lambda: 'read')

    assert authorized_posts.count() == 3
    assert test_data['public_user_post'] in authorized_posts
    assert test_data['private_user_post'] in authorized_posts
    assert test_data['public_manager_post'] in authorized_posts

def test_manages(oso, policy, session, test_data):
    posts = session.query(Post)

    authorized_posts = authorize_query(
        posts,
        lambda: oso,
        lambda: test_data['manager'],
        lambda: 'read')

    assert authorized_posts.count() == 4
    assert test_data['public_user_post'] in authorized_posts
    assert test_data['private_user_post'] in authorized_posts
    assert test_data['public_manager_post'] in authorized_posts
    assert test_data['private_manager_post'] in authorized_posts
