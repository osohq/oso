allow("test_user", "get", post: test_app::Post) if
    post.is_private = false and post.name = "test_public";

allow("test_admin", "get", _: test_app::Post);
