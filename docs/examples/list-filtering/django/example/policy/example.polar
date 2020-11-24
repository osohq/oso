allow(_: example::User, "read", post: example::Post) if
    post.access_level = "public";

allow(user: example::User, "read", post: example::Post) if
    post.access_level = "private" and
    post.creator = user;

allow(user: example::User, "read", post: example::Post) if
    post.access_level = "private" and
    post.creator in user.direct_reports.all();

allow(_: example::User, "read", _: example::User);
