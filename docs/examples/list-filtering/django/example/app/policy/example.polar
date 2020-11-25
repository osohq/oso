allow(_: app::User, "GET", post: app::Post) if
    post.access_level = "public";

allow(user: app::User, "GET", post: app::Post) if
    post.access_level = "private" and
    post.creator = user;

allow(user: app::User, "GET", post: app::Post) if
    post.access_level = "private" and
    post.creator in user.direct_reports.all();
