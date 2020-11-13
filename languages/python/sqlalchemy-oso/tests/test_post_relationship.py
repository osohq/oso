"""Standardized tests for adapters based on the Post model.

Tests come from the relationship document & operations laid out there
https://www.notion.so/osohq/Relationships-621b884edbc6423f93d29e6066e58d16.
"""
import pytest

from sqlalchemy import create_engine
from sqlalchemy.engine import Engine
from sqlalchemy.orm import sessionmaker, relationship
from sqlalchemy.orm.session import Session
from sqlalchemy.schema import Table
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy import Column, Integer, String, Enum, Boolean, ForeignKey

from oso import Oso
from sqlalchemy_oso.auth import authorize_model, register_models

ModelBase = declarative_base(name='ModelBase')

def print_query(query):
    print(query.statement.compile(), query.statement.compile().params)

class Tag(ModelBase):
    __tablename__ = 'tags'

    name = Column(String, primary_key=True)

    # If provided, posts in this tag always have the public access level.
    is_public = Column(Boolean, default=False, nullable=False)

post_tags = Table('post_tags', ModelBase.metadata,
    Column('post_id', Integer, ForeignKey('posts.id')),
    Column('tag_id', Integer, ForeignKey('tags.name'))
)

user_tags = Table('user_tags', ModelBase.metadata,
    Column('user_id', Integer, ForeignKey('users.id')),
    Column('tag_id', Integer, ForeignKey("tags.name"))
)

class Post(ModelBase):
    __tablename__ = 'posts'

    id = Column(Integer, primary_key=True)
    contents = Column(String)
    access_level = Column(Enum('public', 'private'), nullable=False)

    created_by_id = Column(Integer, ForeignKey('users.id'))
    created_by = relationship('User')

    needs_moderation = Column(Boolean, nullable=False, default=False)

    tags = relationship('Tag', secondary=post_tags)

class User(ModelBase):
    __tablename__ = 'users'

    id = Column(Integer, primary_key=True)
    username = Column(String, nullable=False)

    is_moderator = Column(Boolean, nullable=False, default=False)
    is_banned = Column(Boolean, nullable=False, default=False)

    # Single tag
    tag = relationship('Tag')
    tag_name = Column(Integer, ForeignKey('tags.name'))

    # Many tags
    tags = relationship('Tag', secondary=user_tags)

@pytest.fixture
def post_fixtures():
    def create(session: Session):
        foo = User(id=0, username="foo")
        admin_user = User(id=1, username="admin_user", is_moderator=True)
        bad_user = User(id=2, username="bad_user", is_banned=True)
        users = [foo, admin_user, bad_user]

        posts = [
            Post(id=0, contents="foo public post", access_level='public', created_by=foo),
            Post(id=1, contents="foo public post 2", access_level='public', created_by=foo),

            Post(id=3, contents="foo private post", access_level='private', created_by=foo),
            Post(id=4, contents="foo private post 2", access_level='private', created_by=foo),

            Post(id=5, contents="private for moderation", access_level='private',
                 needs_moderation=True, created_by=foo),
            Post(id=6, contents="public for moderation", access_level='public',
                 needs_moderation=True, created_by=foo),

            Post(id=7, contents="admin post", access_level='public',
                 needs_moderation=True, created_by=admin_user),
            Post(id=8, contents="admin post", access_level='private',
                 needs_moderation=True, created_by=admin_user),

            Post(id=9, contents="banned post", access_level='public',
                 created_by=bad_user),
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
    engine = create_engine('sqlite:///:memory:')
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

def test_authorize_model_basic(session, oso, fixture_data):
    """Test that a simple policy with checks on non-relationship attributes is correct."""
    oso.load_str('allow("user", "read", post: Post) if post.access_level = "public";')
    oso.load_str('allow("user", "write", post: Post) if post.access_level = "private";')
    oso.load_str('allow("admin", "read", post: Post);')
    oso.load_str('allow("moderator", "read", post: Post) if '
                 '(post.access_level = "private" or post.access_level = "public") and '
                 'post.needs_moderation = true;')

    posts = authorize_model(oso, "user", "read", session, Post)

    assert posts.count() == 5
    assert posts.all()[0].contents == "foo public post"
    assert posts.all()[0].id == 0

    posts = authorize_model(oso, "user", "write", session, Post)

    assert posts.count() == 4
    assert posts.all()[0].contents == "foo private post"
    assert posts.all()[1].contents == "foo private post 2"

    posts = authorize_model(oso, "admin", "read", session, Post)
    assert posts.count() == 9

    posts = authorize_model(oso, "moderator", "read", session, Post)
    print_query(posts)
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    posts = authorize_model(oso, "guest", "read", session, Post)
    assert posts.count() == 0

def test_authorize_scalar_attribute_eq(session, oso, fixture_data):
    """Test authorization rules on a relationship with one object equaling another."""
    # Object equals another object
    oso.load_str('allow(actor: User, "read", post: Post) if post.created_by = actor and '
                 'post.access_level = "private";')
    oso.load_str('allow(actor: User, "read", post: Post) if '
                 'post.access_level = "public";')
    oso.load_str('allow(actor: User{is_moderator: true}, "read", post: Post) if '
                 'post.access_level = "public";')

    foo = session.query(User).filter(User.username == "foo").first()

    posts = authorize_model(oso, foo, "read", session, Post)
    print_query(posts)

    def allowed(post):
        return (post.access_level == 'public' or
            post.access_level == 'private' and
            post.created_by == foo)

    assert posts.count() == 8
    assert all(allowed(post) for post in posts)

def test_authorize_scalar_attribute_condition(session, oso, fixture_data):
    """Scalar attribute condition checks."""
    # Object equals another object

    oso.load_str('allow(actor: User, "read", post: Post) if post.created_by.is_banned = false and '
                 'post.created_by.username = actor.username and post.access_level = "private";')

    oso.load_str('allow(actor: User, "read", post: Post) if post.created_by.is_banned = false and '
                 'post.access_level = "public";')

    # moderator can see posts made by banned users.
    oso.load_str('allow(actor: User, "read", post: Post) if actor.is_moderator = true and post.created_by.is_banned = true;')

    foo = session.query(User).filter(User.username == "foo").first()

    posts = authorize_model(oso, foo, "read", session, Post)

    def allowed(post, user):
        return ((post.access_level == 'public' and post.created_by.is_banned == False) or
            post.access_level == 'private' and
            post.created_by == user)

    assert posts.count() == 7
    assert all(allowed(post, foo) for post in posts)

    admin = session.query(User).filter(User.username == "admin_user").first()
    posts = authorize_model(oso, admin, "read", session, Post)

    def allowed_admin(post):
        return post.created_by.is_banned

    assert posts.count() == 6
    for post in posts:
        assert allowed(post, admin) or allowed_admin(post)


@pytest.fixture
def tag_test_fixture(session):
    """Test data for tests with tags."""
    user = User(username='user')
    other_user = User(username='other_user')
    moderator = User(username='moderator', is_moderator=True)

    eng = Tag(name="eng")
    random = Tag(name="random", is_public=True)

    user_public_post = Post(contents="public post", created_by=user, access_level="public")
    user_private_post = Post(contents="private user post", created_by=user, access_level="private")

    other_user_public_post = Post(contents="other user public", created_by=other_user,
                                  access_level="public")
    other_user_private_post = Post(contents="other user private", created_by=other_user,
                                  access_level="private")
    other_user_random_post = Post(contents="other user random", created_by=other_user,
                                  access_level="private",
                                  tags=[random])

    # HACK!
    objects = {}
    for (name, local) in locals().items():
        if name != "session" and name != "objects":
            session.add(local)

        objects[name] = local

    session.commit()

    return objects

def test_in_multiple_attribute_relationship(session, oso, tag_test_fixture):
    oso.load_str("""
        allow(user, "read", post: Post) if post.access_level = "public";
        allow(user, "read", post: Post) if post.access_level = "private" and post.created_by = user;
        allow(user, "read", post: Post) if tag in post.tags and tag.is_public = true;
    """)

    posts = authorize_model(oso, tag_test_fixture['user'], "read", session, Post)
    print(str(posts.statement.compile()))
    assert tag_test_fixture['user_public_post'] in posts
    assert tag_test_fixture['user_private_post'] in posts
    assert tag_test_fixture['other_user_public_post'] in posts
    assert not tag_test_fixture['other_user_private_post'] in posts
    assert tag_test_fixture['other_user_random_post'] in posts

# TODO test non Eq conditions
# TODO test f(x) if not x.boolean_attr;
