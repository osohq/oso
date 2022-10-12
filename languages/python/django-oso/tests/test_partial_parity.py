"""Partial parity tests based on

https://www.notion.so/osohq/Supported-Query-Types-and-Features-435d7a998dc14db3a125c6e5ba5fe6ba.
"""
import pytest
from test_app2.models import Post, Tag

from django_oso.oso import Oso, reset_oso


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.mark.xfail(reason="Not supported yet.")
@pytest.mark.django_db
def test_field_comparison(load_additional_str):
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="private post", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    post0.save()
    post1.save()
    post2.save()

    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            post.title = post.contents;
    """
    )

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post1 in posts
    assert post2 in posts


@pytest.mark.django_db
def test_scalar_in_list(load_additional_str):
    post0 = Post(id=0, contents="private post", title="not private post")
    post1 = Post(id=1, contents="allowed posts", title="private post")
    post2 = Post(id=2, contents="post", title="post")

    post0.save()
    post1.save()
    post2.save()

    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            post.contents in ["post", "allowed posts"];
    """
    )

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post1 in posts
    assert post2 in posts


@pytest.mark.django_db
def test_ground_object_in_collection(load_additional_str):
    tag = Tag(name="tag")
    post0 = Post(id=0, contents="tag post")
    post1 = Post(id=1, contents="no tag post")
    post2 = Post(id=2, contents="tag 2 post")

    tag.save()
    post0.save()
    post1.save()
    post2.save()

    post0.tags.set([tag])
    post2.tags.set([tag])

    Oso.register_constant(tag, "allowed_tag")
    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            allowed_tag in post.tags;
    """
    )

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post0 in posts
    assert post2 in posts


@pytest.mark.xfail(reason="Negate in not supported yet.")
@pytest.mark.django_db
def test_all_objects_collection_condition(oso, engine, load_additional_str):
    public_tag = Tag(name="public", is_public=True)
    private_tag = Tag(name="private", is_public=False)

    post0 = Post(id=0, contents="public tag", tags=[public_tag])
    post1 = Post(id=1, contents="no tags", tags=[])
    post2 = Post(id=2, contents="both tags", tags=[public_tag, private_tag])
    post3 = Post(id=3, contents="public tag 2", tags=[public_tag])
    post4 = Post(id=4, contents="private tag", tags=[private_tag])

    public_tag.save()
    private_tag.save()
    post0.save()
    post1.save()
    post2.save()
    post3.save()
    post4.save()

    post0.tags.set([public_tag])
    post2.tags.set([public_tag, private_tag])
    post3.tags.set([public_tag])
    post4.tags.set([private_tag])

    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            forall(tag in post.tags, tag.is_public = true);
    """
    )

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post0 in posts
    assert post3 in posts


@pytest.mark.xfail(reason="Negate in not supported yet.")
@pytest.mark.django_db
def test_no_objects_collection_condition(load_additional_str):
    public_tag = Tag(name="public", is_public=True)
    private_tag = Tag(name="private", is_public=False)

    post0 = Post(id=0, contents="public tag", tags=[public_tag])
    post1 = Post(id=1, contents="no tags", tags=[])
    post2 = Post(id=2, contents="both tags", tags=[public_tag, private_tag])
    post3 = Post(id=3, contents="public tag 2", tags=[public_tag])
    post4 = Post(id=4, contents="private tag", tags=[private_tag])

    public_tag.save()
    private_tag.save()
    post0.save()
    post1.save()
    post2.save()
    post3.save()
    post4.save()

    post0.tags.set([public_tag])
    post2.tags.set([public_tag, private_tag])
    post3.tags.set([public_tag])
    post4.tags.set([private_tag])

    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            not (tag in post.tags and tag.is_public = true);
    """
    )

    posts = Post.objects.authorize(None, actor="u", action="r").all()
    assert len(posts) == 2
    assert post0 in posts
    assert post3 in posts
