---
title: "Request-level Enforcement"
weight: 40
any: true
description: >
  Learn about enforcing request-level authorization, controlling who can access
  which routes or endpoints in your app.
---

# Request-level Enforcement

It's likely that your app already has a way of preventing logged-out users from
accessing certain endpoints. This is an example of **"request-level"
enforcement**, and you can use Oso to express these types of rules.

Often this gets more complicated as time goes along. "Logged-in" vs "logged-out"
is the first question you might want to ask, but perhaps users can't perform
certain actions until they've verified their email, or perhaps you allow users
to create access tokens with particular scopes that limit which endpoints are
accessible.

To implement "request-level" enforcement, you can use the `{{< exampleGet "authorizeRequest" >}}`
method provided by your Oso instance.

In your request middleware, after you've loaded the user, but not executed any
endpoint-specific code, you should call Oso to determine whether the action is
allowed:

{{% exampleGet "exampleCall" %}}

You can see this method only takes two arguments: user and request. The type of
the `request` argument depends on your app. In most frameworks, there is a
`Request` type, and we recommend using that so your policy can access fields
such as its method (POST, GET, etc) and its path.

If you'd prefer, you can also pass a string as the `request` argument. The
string could hold the request's path or, if you're using GraphQL, the string
could be the name of the current GraphQL query or mutation.

Writing a request-level authorization policy means implementing an
`allow_request` rule like this:

{{% exampleGet "simplePolicy" %}}

## Authorization Failure

What happens when the authorization fails? That is, what if there is not an
`allow_request` rule that gives the user permission to perform the given
request?

In that case, the `{{< exampleGet "authorizeRequest" >}}` method raises a {{% exampleGet "forbiddenErrorLink" %}},
which your app should handle by returning a 403 error to the user and aborting
the request.

## Access token scopes

You can use a request-level authorization policy to protect your endpoints using
access token scopes. This can be used to build features like Github's [OAuth
Scopes](https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps).


```polar
allow_request(token: AccessToken, _: Request{method: "POST", path: "/repos"}) if
    "repos.create" in token.scopes;

allow_request(token: AccessToken, _: Request{method: "GET", path: "/repos"}) if
    "repos.list" in token.scopes;
```

To do this, you'd have to call `{{< exampleGet "authorizeRequest" >}}` with the access token
used to make the request:

{{% exampleGet "exampleCallWithAccessToken" %}}
