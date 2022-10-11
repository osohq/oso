import pytest
from oso import Oso
from polar.exceptions import DuplicateClassAliasError, OsoError
from sqlalchemy import Column, Integer, String, create_engine
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso.auth import register_models
from sqlalchemy_oso.session import authorized_sessionmaker

from .conftest import print_query

Base = declarative_base(name="Base")


class Post(Base):
    __tablename__ = "posts_two"

    id = Column(Integer, primary_key=True)
    title = Column(String, nullable=False)


def test_duplicate_models(oso):
    from .duplicate_model import Post as DuplicatePost

    try:  # SQLAlchemy 1.4
        engine = create_engine("sqlite:///:memory:", enable_from_linting=False)
    except TypeError:  # SQLAlchemy 1.3
        engine = create_engine("sqlite:///:memory:")
    Base.metadata.create_all(engine)

    with pytest.raises(OsoError):
        oso = Oso()
        register_models(oso, Base)

    oso = Oso()
    oso.register_class(DuplicatePost, name="duplicate::Post")
    register_models(oso, Base)

    for m in [Post, DuplicatePost]:
        with pytest.raises(DuplicateClassAliasError):
            oso.register_class(m)

    oso.load_str(
        """
        allow(_, "read", post: duplicate::Post) if
            post.admin;

        allow(_, "read", post: Post) if
            post.title = "Test";
    """
    )

    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: "user",
        get_checked_permissions=lambda: {Post: "read", DuplicatePost: "read"},
        bind=engine,
    )

    session = Session()

    session.add(Post(id=1, title="Test"))
    session.add(Post(id=2, title="Not Test"))
    session.add(DuplicatePost(id=3, admin=True))
    session.add(DuplicatePost(id=4, admin=False))

    session.commit()

    posts = session.query(Post)
    print_query(posts)

    assert posts.count() == 1
    assert posts[0].id == 1

    posts = session.query(DuplicatePost)
    print_query(posts)
    assert posts.count() == 1
    assert posts[0].id == 3
