allow(_: User, "read", post: Post) if
    post.access_level = "public";

allow(user: User, "read", post: Post) if
    post.access_level = "private" and
    post.created_by = user;

allow(user: User, "read", post: Post) if
    post.access_level = "private" and
    post.created_by in user.manages;

allow(_: User, "read", _: User);
