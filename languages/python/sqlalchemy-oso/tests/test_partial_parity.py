from sqlalchemy.orm import Session

from sqlalchemy_oso.session import AuthorizedSession

from .models import User, Post


def test_field_comparison(oso, engine):
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="private post", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    session = Session(bind=engine)
    session.add_all([post0, post1, post2])
    session.commit()

    oso.load_str("""
        allow(_, _, post: Post) if
            post.title = post.contents;
    """)

    authz_session = AuthorizedSession(oso=oso, user="u", action="a")
    posts = authz_session.query(Post).all()
    post_ids = [p.id for p in posts]

    assert len(posts) == 2
    assert 1 in post_ids
    assert 2 in post_ids
