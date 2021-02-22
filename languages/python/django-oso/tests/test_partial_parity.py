"""Partial parity tests based on

https://www.notion.so/osohq/Supported-Query-Types-and-Features-435d7a998dc14db3a125c6e5ba5fe6ba.
"""
import pytest

from django_oso.models import authorize_model
from django_oso.oso import Oso, reset_oso
from test_app2.models import Post, Tag, User


@pytest.fixture(autouse=True)
def reset():
    reset_oso()

@pytest.mark.django_db
def test_field_comparison():
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="private post", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    post0.save()
    post1.save()
    post2.save()

    Oso.load_str("""
        allow(_, _, post: test_app2::Post) if
            post.title = post.contents;
    """)

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post1 in posts
    assert post2 in posts
