---
title: "Filter Data (Preview)"
weight: 1
showContentForAnyLanguage: true
---

# New Data Filtering for Ruby

This article is about the new data filtering configuration API preview in the
Ruby library.  For other languages, or for general information about data
filtering and how to use it, including how to define relations, please see
[here][docs] instead.

## Filter/Adapter Interface

This API is under active development and should be regarded as unstable where
noted. The two core types are `Filter` and `Adapter`.

To use the new data filtering engine, call `Oso#data_filtering_adapter=` with an
adapter object that provides those two methods.

We supply an example adapter for `ActiveRecord` as part of the preview release.

## Implementing an Adapter

### Adapters

An adapter needs to provide two methods.

#### `{{% exampleGet "executeQuery" %}}`

`{{% exampleGet "executeQuery" %}}` takes a query object and returns a list of 
resources. Depending on your ORM, this could be as simple as `query.to_a`.

#### `{{% exampleGet "buildQuery" %}}`

`{{% exampleGet "buildQuery" %}}` takes a `Filter` and returns a query object, for
example, an `ActiveRecord::Relation`.

A `Filter` is an abstract representation of a database query that comes from Oso.
It currently has four fields: `root`, `relations`, `conditions`, and `types`.

The `{{% exampleGet "buildQuery" %}}` method's job is to create a query over the
set of tables implied by the `relations`, for those records matching the
`conditions`. The `conditions` represent an `OR` of `AND`s, so a record matches if
for any top level list every condition triple it contains is true for that record.

##### Filters

The four attributes of a `Filter` are:
- `root`: the class of the queried resource
- `relations`: a list of `[from_class, rel_name, to_class]` triples,
  with values corresponding to your globally registered relations. We apologize
  for the number of different things we designate a `relation`. This will be
  addressed in an upcoming release.
- `conditions`: a list of lists of `[lhs, op, rhs]` triples, with `op` in
  `%w[Eq Neq In Nin]`, and `lhs` and `rhs` being `Projection` objects having a
  `source` (a resource class) and a `field` (an optional string representing a
  column name, or `nil` indicating "object identity" according to your case
  (for example, compare by a primary key).
- `types`: a hash whose keys are class objects and their names and registered
  aliases, and whose values are Oso-defined type information needed to understand
  the `Filter`'s `relations`. This attribute may be removed in an
  upcoming release.


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

## Limitations

Like previous releases, the new data filtering can't perform method calls on
unbound variables, and excludes Polar operators and expressions. See the [general
data filtering docs][docs] for more details.

This release doesn't include support for queries where a single resource occurs
more than once. Therefore direct or indirect relations from a type to itself are
prohibited. We plan to remove this limitation in an upcoming release.

[docs]: https://docs.osohq.com/guides/data_filtering.html
