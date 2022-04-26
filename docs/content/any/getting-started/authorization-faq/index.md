---
title: Authorization FAQ
metaTitle: Authorization FAQ
description: |
  We'll answer your common questions about authorization!
any: false
weight: 1
---

# Authorization FAQ


## What’s the difference between authentication and authorization?

  Authentication is the mechanism for verifying who a user is.
  A username and password, for instance, verify that you can log in to an account.

  Authorization is the mechanism for controlling what a user can do.
  For instance: after you’ve logged in, do you have access to a particular private git repository on GitHub?
  The answer is determined by your authorization system.

## What is a permission?

A permission grants the ability to perform an action on a particular resource.
For instance—you have permission to *read* Oso’s code on GitHub (we’re open source!).
That means you have the *read* permission for the resource `github.com/osohq/oso`.
On the other hand, you can't *push* directly to our source (please open a PR instead).
In that case, you don't have the *push* permission for that resource.

To make it easy to grant permissions, systems usually bundle permissions into groups.
Role-based access control, relationship-based access control, and attribute-based access control are ways to assign groups of permissions.

## What is a role, and what is role-based access control (RBAC)?

Roles are groups of permissions that are assigned to users.
Nearly every app uses roles! You’ve probably seen Member, Manager, and Admin roles in the wild.
Users with the Member role may only have *read* permission for resources they created, but users with the Admin role might have *read* permission for every resource.
Users can have many roles in a single app—you might have an Owner role for repositories you create, but the Member role in your employer’s repositories.

## What is a relationship, and what is relationship-based access control (ReBAC)?

Relationship-based access control means determining permissions based on the relationships between your app's objects.
For instance, the authorization rule “you can read an issue if you are a contributor of the issue’s parent repository” depends on a parent-child relationship between resources.

## What is an attribute, and what is attribute-based access control (ABAC)?

Attribute-based access control is a way to control access to a resource depending on its attributes.
For instance, the policy that “anybody can read a repository if it is marked public” depends on the repository’s “public” attribute.

## What is an access control list (ACL)?

An access control list (an ACL, often pronounced “ackel”) is a list of permissions attached to a resource.
For instance, a repository may have a specific list of contributors and their permissions: “Alice: pull, push; Bob: pull; Carlos: pull.” ACLs can also include relations: “user Alice has relation Owner to object Repository.”

## What is Google Zanzibar?

Zanzibar is Google’s internal, low-level authorization service.
It’s a centralized service that provides an authorization API for Google’s application teams.
Google has published a detailed description of the architecture, so there are several open- and closed-source re-implementations.
Zanzibar-like systems are heavyweight authorization tools with high setup costs.
Whether you should use Zanzibar-likes or not depends on your application architecture.

If you’re curious about the precise details, we’ve walked through building [Zanzibar from scratch](https://www.osohq.com/post/zanzibar) to show you how it works!

## How should I implement authorization in my microservice architecture?

It's always best to build authorization around your existing infrastructure.

If you already maintain a very performant way of exchanging data, like gRPC, you can use that to share authorization data between services.

If not, the simplest approach is to put your authorization data (like roles) in a JSON Web Token.
This approach works well, but tokens have size limits.
Your app may outgrow web tokens quickly.

The second-simplest method—and the most robust—is to use an authorization service like Oso Cloud.
We go over each of your options in detail in our article on [Why Authorizaion is Hard](https://www.osohq.com/post/why-authorization-is-hard).

## What’s the best way to model my authorization logic?

Instead of scattering authorization throughout your code, it's best to move your authorization logic into a policy.
That policy will define the resources you’re controlling access to and the rules governing that access.
When you need to make an authorization decision, you'll use your policy to make that decision.

To start, define your policy using [role-based authorization](https://www.osohq.com/academy/role-based-access-control-rbac) (RBAC).
Roles will fit most of the cases you'll run into in an app.

When you want to write authorization logic that depends on the existing relationships in your application, like granting access to repositories if you can access the parent organization, you can mix in [relationship-based access control](https://www.osohq.com/academy/relationship-based-access-control-rebac) ReBAC).
Relationships will cover nearly every common authorization case.

If access to a resource depends on an attribute that isn’t a role or relationship—for example, an `is_public` flag—you can write authorization rules based on arbitrary attributes of your resources.

## How should I implement authorization in GraphQL?

In GraphQL, build your authorization logic as close to the data as possible.
Usually, this means putting your authorization in the GraphQL API itself.

Putting your authorization logic at GraphQL's data access layer is very close to the data itself.
That's the best way to authorize read requests, but that code may not have enough context to deal with authorization for write requests.

If you have a small application with basic authorization requirements, build your authorization in GraphQL resolvers.
In a resolver, you'll have the context you need—the actor, the action, and the resource—to write your authorization logic correctly.
However, you won’t be able to reuse your authorization code between resolvers.

If you find yourself writing authorization logic in every resolver, consider moving your authorization to a GraphQL directive.
Using directives will let you reuse authorization code throughout your schema.

In distributed or federated GraphQL, there are a whole new set of problems! We cover many of them in our guide to [authorization patterns in GraphQL](https://www.osohq.com/post/graphql-authorization).
