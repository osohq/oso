# Polar Overview

## What does Polar need to do?

Polar is built for expressing authorization logic as code.

There are a few patterns exhibited in authorization logic that are important to support:

### Logical branching

Authorization logic has a _ton_ of branching logic. This is the crux of why logic
programming is a good fit.

A GitClub user is allowed to close an issue if the following is true:
- The issue belongs to a repository
- AND
  - The user has a role for that repository
  - AND
    - The role is "writer"
- OR
  - The repository belongs to a organization
  - AND
    - The user has a role for that organization
    - AND
      - the role is "admin"
    - OR
      - the role is "member"

### Reusable logic

As in the the above, it's normal to have the same kinds of conditions and checks
repeated in multiple different places.

- user has a role for <resource>
- the role is <name>
- <child resource> belongs to <parent resource>

### Attribute checks

It's also common to need to check attributes on data in order to make authorization
decisions.

For example:

- the user has the "superadmin" attribute
- the resource has the "public" attribute


### Multiple question types

The core authorization check is: can <user> perform <action> on <resource>,
where user, action, and, resource all have concrete values.

But it can be helpful to ask variations of these:

> What are all the users that can perform <action> on <resource>

Can be useful as part of a UI, to help users understand who can see a document,
or for auditing of access control.

> What are all the resources this <actor> can perform <action> on?

Important for any application that needs to return an index of resources, filtered
down to only those a user is allowed to see (action="read").

## Paradigms that fit the above

Logical programming fits the above requirements to a tee.

But which variant specifically?

## Prolog

- Turing complete
- General purpose programming
- Meta programming
- Suitable for _writing_ DSls
- Extensions for doing constraint programming

## Datalog

- Limited negation, recursion
- Constraints?

## Mini-Kanren

- Typically used as an embedded logic language
- Relational programming
- Many variants working with constraints