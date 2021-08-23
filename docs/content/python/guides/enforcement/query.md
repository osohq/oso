---
title: "Query-level Enforcement"
weight: 20
any: true
draft: true
---

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Query-level Enforcement

- Once you've secured your endpoints with resource-level enforcement, you might find that list endpoints throw a wrench into the works. When an endpoint is returning _many_ resources to a user, we need to somehow make sure that the user can access each one.
- One naive way of doing this is by performing resource-level enforcement on each object before it's returned to the user. This can work in simple cases, but doesn't work when the user can only access a small percentage of the objects in question.
- The solution is query-level enforcement, which lets you authorize a query _before_ it's executed. Imagine we have a simple policy for read access to articles:

  ```ruby
  # Anybody can read published articles
  allow(_, "read", article: Article) if
  		article.is_published;

  # Users can read their own articles (even if unpublished)
  allow(user, "read", article: Article) if
  		article.user_id = user.id;
  ```

- Enforcing this policy at the query level means adding a condition to your article queries (we'll use SQL as an example):

  ```ruby
  SELECT * FROM articles WHERE
  	(articles.is_published OR articles.user_id = ?) # Query-level enforcement
  	AND ... # Whatever else we're filtering by
  ```

- Because your Oso policies are declarative, it can be used to add these conditions automatically.
- Oso's method to perform query-level enforcement is called `authorize_query`, and it takes a user and a model class as arguments:

  ```ruby
  oso.authorize_query(current_user, Expense)
  ```

- Exactly what's returned from this method depends on the enforcer you're using.
  We have enforcers for several popular ORMs which return "query sets" with
  proper filters applied. An example of using `authorize_query` in an app:

  ```ruby
  def list_articles_for_org(user, org):
  		query = oso.authorize_query(user, Article)

  		# Apply more filters to the query and fetch the data
  		articles = query.filter(org_id=org.id).all()
  		return articles

  ```

- You can also write your own enforcers if you're not using an ORM, or are using one without official support from Oso. TODO: Link to a guide →
