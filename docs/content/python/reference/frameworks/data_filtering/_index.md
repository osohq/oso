---
title: Data Filtering Integrations
weight: 2
description: Learn how to filter data collections based on authorization logic with Oso.
---

# Filter Data Collections

Oso supports applying authorization logic at the ORM layer so that you can
efficiently authorize entire data sets. For example, suppose you have millions
of posts in a social media application created by thousands of users, and
regular users are only authorized to view posts from their friends. It would be
inefficient to fetch all of the posts and authorize them one by one. It would
be much more efficient to distill from the policy a _filter_ that can be
applied by the ORM to return only the authorized posts. This idea can be used
in any scenario where you need to authorize a subset of a large collection of
data.

The Oso policy engine can now produce such filters from your policy with our
Django & SQLAlchemy integrations.
For more information on how data filtering works, head over to our explanation of Oso's
[internals](project/internals#data-filtering).

To get started with data filtering in Django or SQLAlchemy, follow one of the guides below.


