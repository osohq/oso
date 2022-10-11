---
title: "Filter Data"
weight: 1
showContentForAnyLanguage: true
---

# Filter Data

{{% ifLang "rust" %}}

{{% callout "Coming soon" %}}

Data filtering is coming soon for Rust!

If you want to register your interest for Data Filtering in
Rust, [drop into our Slack](https://join-slack.osohq.com).

Below is the documentation for Data Filtering in Python.

{{% /callout %}}

{{% /ifLang %}}

{{% ifLang "java" %}}

{{% callout "Coming soon" %}}

Data filtering is coming soon for Java!

If you want to register your interest for Data Filtering in
Java, [drop into our Slack](https://join-slack.osohq.com).

Below is the documentation for Data Filtering in Python.

{{% /callout %}}

{{% /ifLang %}}

This guide shows you how to use **Data Filtering** in the Oso Library. Data
filtering lets you select certain data from your data store, based on the logic
in your policy. In the Oso Library, data filtering works by telling Oso how to turn Polar
constraints into queries against your data store, such as SQL queries or ORM
query objects.

If you're using Oso Cloud as an authorization service, data filtering is built in. Read about how to [list
authorized resources using the Oso Cloud API](https://www.osohq.com/docs/guides/filter-data).

## Why do you need Data Filtering?

When you call `authorize(actor, action, resource)` , Oso evaluates the allow
rule(s) you have defined in your policy to determine if `actor` is allowed
to perform `action` on `resource`.  For example, if `jane` wants to `"edit"`
a `document`, Oso may check that `jane = document.owner`.  But what if you
need the set of all documents that Jane is allowed to edit?  For example, you
may want to render them as a list in your application.

One way to answer this question is to take every document in the system and
call `is_allowed` on it. This isn’t efficient and many times is just
impossible. There could be thousands of documents in a database but only three
that have the owner `"steve"`. Instead of fetching every document and passing
it into Oso, it's better to ask the database for only the documents that
have the owner `"steve"`. Oso provides a "data filtering" API to do this.

You can use data filtering to enforce authorization on queries made to your data
store. Oso will take the logic in the policy and turn it into a query for the
authorized data. Examples could include an ORM filter object, an HTTP request or
an elastic-search query. The query object and the way the logic maps to a query
are both user defined.

Data filtering is initiated through two methods on `Oso`.

`{{% exampleGet "authorizedResources" %}}` returns a list of all the
resources a user is allowed to do an action on. The results of a built and
executed query.

`{{% exampleGet "authorizedQuery" %}}` returns the query object itself.
This lets you add additional filters or sorts or any other data to it before
executing it.

The mapping from Polar to a query is defined by an `Adapter`. If an adapter exists
for your ORM or database you can use it, otherwise you may have to implement your own.

## Implementing an Adapter

### Adapters

An adapter is an interface that defines two methods. Once you've defined an adapter, you
can configure your Oso instance to use it with the
`{{% exampleGet "setDataFilteringAdapter" %}}` method.

#### Build a Query

`{{% exampleGet "buildQuery" %}}` takes some type information and a `Filter` object and returns a `Query`.

A `{{% exampleGet "filterName" %}}` is a representation of a query. It is very similar to a SQL query.
It has four fields:

- `{{% exampleGet "filterRoot" %}}` Is the name of the type we are filtering.
- {{% exampleGet "filterRelations" %}} Are named relations to other types, typically turned into joins.
- `{{% exampleGet "filterConditions" %}}` Are the individual pieces of logic that must be true with respect to objects
  matching the filter. These typically get turned into where clauses.
- `{{% exampleGet "filterTypes" %}}` Is a map from type names to user type information, including registered relations.
  We use this to generate the join SQL.

##### Relations

A relation has three properties: `{{% exampleGet "relationFrom" %}}`, `{{% exampleGet "relationName" %}}`, and `{{% exampleGet "relationTo" %}}`.
The adapter uses these properties to look up the tables and fields to join together for
the query.

##### Conditions

A condition has three properties `{{% exampleGet "conditionLeft" %}}`, `{{% exampleGet "conditionOp" %}}`, and `{{% exampleGet "conditionRight" %}}`.
The left and right fields will be either `Immediate` objects with a `{{% exampleGet "immediateValue" %}}` field that can
be inserted directly into a query, or `Projection` objects with string properties
`{{% exampleGet "projectionType" %}}` and optionally `{{% exampleGet "projectionField" %}}`. A
missing `{{% exampleGet "projectionField" %}}` property indicates the adapter should substitute
an appropriate unique identifier, usually a primary key.

#### Execute a Query

`{{% exampleGet "executeQuery" %}}` takes a query and returns a list of the results.

### Fields

The other thing you have to provide to use data filtering is type information
for registered classes. This lets Oso know what the types of an object's fields
are. Oso needs this information to handle specializers and other things in the
policy when we don't have a concrete resource. The fields are a
{{% exampleGet "map" %}} from field name to type.

## Relations

Often you need data that is not contained on the object to make
authorization decisions. This comes up when the role required to
do something is implied by a role on it's parent object. For instance,
you want to check the organization for a repository but that data isn't
embedded on the repository object. You can add a `Relation` type to the type
definition that states how the other resource is related to this one. Then
you can access this field in the policy like any other field and it will
fetch the data when it needs it (via the query functions).

`Relation`s are a special type that tells Oso how one Class is related to
another. They specify what the related type is and how it's related.

- `kind` is either "one" or "many". "one" means there is one related object and
      "many" means there is a list of related objects.
- `other_type` is the class of the related objects.
- `my_field` Is the field on this object that matches `other_field`.
- `other_field` Is the field on the other object that matches `this_field`.

The `my_field` / `other_field` relationship is similar to a foreign key. It lets Oso
know what fields to match up with building a query for the other type.

## Example

{{< literalInclude
      dynPath="exampleBPath"
      from="docs: begin-b1"
      to="docs: end-b1"
      fallback="no" >}}

{{< literalInclude
      dynPath="exampleBPath"
      from="docs: begin-b2"
      to="docs: end-b2"
      fallback="no" >}}

{{< literalInclude
      dynPath="policyBPath"
      fallback="no" >}}

{{< literalInclude
      dynPath="exampleBPath"
      from="docs: begin-b3"
      to="docs: end-b3"
      fallback="no" >}}

## Evaluation
When Oso is evaluating data filtering methods it uses the adapter to build queries
and execute them.

Relation fields also work when you are not using data filtering methods are also
use the adapter to query for the related resources when you access them.


## Limitations

Some Polar operators including `cut` and arithmetic operators aren't supported in
data filtering queries.

You can't call any methods on the resource argument or pass the resource as an
argument to other methods. Many cases where you would want to do this are better
handled by `Relation` fields.

The new data filtering backend doesn't support queries where a given resource
type occurs more than once, so direct or indirect relations from a type to itself
are currently unsupported. This limitation will be removed in an upcoming release.

