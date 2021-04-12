# Anyone may view public posts.
allow(_: app::User, "GET", post: app::Post) if
    post.access_level = "public";

# Users may view their own private posts.
allow(user: app::User, "GET", post: app::Post) if
    post.access_level = "private" and
    post.creator = user;

# Users may view private posts created by users who they manage.
allow(user: app::User, "GET", post: app::Post) if
    post.access_level = "private" and
    post.creator in user.direct_reports.all();
