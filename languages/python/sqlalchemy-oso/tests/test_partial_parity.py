import pytest
from sqlalchemy.orm import Session

from sqlalchemy_oso.session import AuthorizedSession

from .models import Post, Tag


@pytest.mark.xfail(reason="Not supported yet.")
def test_field_comparison(oso, engine):
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="private post", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    session = Session(bind=engine)
    session.add_all([post0, post1, post2])
    session.commit()

    oso.load_str(
        """
        allow(_, _, post: Post) if
            post.title = post.contents;
    """
    )

    authz_session = AuthorizedSession(
        oso=oso, user="u", checked_permissions={Post: "a"}, bind=engine
    )
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 1 in post_ids
    assert 2 in post_ids


def test_scalar_in_list(oso, engine):
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="allowed posts", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    session = Session(bind=engine)
    session.add_all([post0, post1, post2])
    session.commit()

    oso.load_str(
        """
        allow(_, _, post: Post) if
            post.contents in ["post", "allowed posts"];
    """
    )

    authz_session = AuthorizedSession(
        oso=oso, user="u", checked_permissions={Post: "a"}, bind=engine
    )
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 1 in post_ids
    assert 2 in post_ids


def test_ground_object_in_collection(oso, engine):
    tag = Tag(name="tag")
    post0 = Post(id=0, contents="tag post", tags=[tag])
    post1 = Post(id=1, contents="no tag post", tags=[])
    post2 = Post(id=2, contents="tag 2 post", tags=[tag])

    session = Session(bind=engine)
    session.add_all([tag, post0, post1, post2])
    session.commit()

    oso.register_constant(tag, "allowed_tag")
    oso.load_str(
        """
        allow(_, _, post: Post) if
            allowed_tag in post.tags;
    """
    )

    authz_session = AuthorizedSession(
        oso=oso, user="u", checked_permissions={Post: "a"}, bind=engine
    )
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 0 in post_ids
    assert 2 in post_ids


@pytest.mark.xfail(reason="Negate in not supported yet.")
def test_all_objects_collection_condition(oso, engine):
    public_tag = Tag(name="public", is_public=True)
    private_tag = Tag(name="private", is_public=False)

    post0 = Post(id=0, contents="public tag", tags=[public_tag])
    post1 = Post(id=1, contents="no tags", tags=[])
    post2 = Post(id=2, contents="both tags", tags=[public_tag, private_tag])
    post3 = Post(id=3, contents="public tag 2", tags=[public_tag])
    post4 = Post(id=4, contents="private tag", tags=[private_tag])

    session = Session(bind=engine)
    session.add_all([public_tag, private_tag, post0, post1, post2, post3, post4])
    session.commit()

    oso.load_str(
        """
        allow(_, _, post: Post) if
            forall(tag in post.tags, tag.is_public = true);
    """
    )

    authz_session = AuthorizedSession(
        oso=oso, user="u", checked_permissions={Post: "a"}, bind=engine
    )
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 0 in post_ids
    assert 3 in post_ids


@pytest.mark.xfail(reason="Negate in not supported yet.")
def test_no_objects_collection_condition(oso, engine):
    public_tag = Tag(name="public", is_public=True)
    private_tag = Tag(name="private", is_public=False)

    post0 = Post(id=0, contents="public tag", tags=[public_tag])
    post1 = Post(id=1, contents="no tags", tags=[])
    post2 = Post(id=2, contents="both tags", tags=[public_tag, private_tag])
    post3 = Post(id=3, contents="public tag 2", tags=[public_tag])
    post4 = Post(id=4, contents="private tag", tags=[private_tag])

    session = Session(bind=engine)
    session.add_all([public_tag, private_tag, post0, post1, post2, post3, post4])
    session.commit()

    oso.load_str(
        """
        allow(_, _, post: Post) if
            not (tag in post.tags and tag.is_public = true);
    """
    )

    authz_session = AuthorizedSession(
        oso=oso, user="u", checked_permissions={Post: "a"}, bind=engine
    )
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 1 in post_ids
    assert 4 in post_ids
