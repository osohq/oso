from pathlib import Path

import pytest

from sqlalchemy import create_engine
from sqlalchemy.orm import Session

from oso import Oso
from sqlalchemy_oso.session import scoped_session
from sqlalchemy_oso.auth import register_models

from .models import Post, User, Model

POLICY_FILE = Path(__file__).absolute().parent / 'policy.polar'

@pytest.fixture
def engine():
    engine = create_engine('sqlite:///:memory:')
    Model.metadata.create_all(engine)
    return engine

@pytest.fixture
def authorization_data():
    return {
        'user': None,
        'action': "read"
    }

@pytest.fixture
def session(engine, oso, authorization_data):
    return scoped_session(
        bind=engine,
        get_oso=lambda: oso,
        get_user=lambda: authorization_data['user'],
        get_action=lambda: authorization_data['action']
    )

@pytest.fixture
def oso():
    return Oso()

@pytest.fixture
def policy(oso):
    register_models(oso, Model)
    oso.load_file(POLICY_FILE)

@pytest.fixture
def test_data(engine):
    session = Session(bind=engine, expire_on_commit=False)

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

def test_basic(oso, policy, session, test_data, authorization_data):
    authorization_data['user'] = test_data['user']
    posts = session.query(Post)

    assert posts.count() == 3
    posts = [p.id for p in posts.all()]
    assert test_data['public_user_post'].id in posts
    assert test_data['private_user_post'].id in posts
    assert test_data['public_manager_post'].id in posts

def test_manages(oso, policy, session, test_data, authorization_data):
    authorization_data['user'] = test_data['manager']
    posts = session.query(Post)

    assert posts.count() == 4
    posts = [p.id for p in posts.all()]
    assert test_data['public_user_post'].id in posts
    assert test_data['private_user_post'].id in posts
    assert test_data['public_manager_post'].id in posts
    assert test_data['private_manager_post'].id in posts

def test_user_access(oso, policy, session, test_data, authorization_data):
    authorization_data['user'] = test_data['user']
    users = session.query(User)
    assert users.count() == 2
