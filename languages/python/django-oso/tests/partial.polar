allow("test_user", "get", post: test_app::Post) if
    post.is_private = false and post.timestamp > 0;

allow("test_admin", "get", _: test_app::Post);
