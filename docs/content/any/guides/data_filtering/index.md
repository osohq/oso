---
title: "Filter Data"
weight: 1
showContentForAnyLanguage: true
---
# Data filtering

When you call `authorize(actor, action, resource)` , Oso evaluates the allow
rule(s) you have defined in your policy to determine if that `actor` is allowed to do that `action` on that `resource`.

If you have a policy like this,

```polar
allow(actor, "get", doc: Document) if doc.owner = actor;
```

Oso checks that the passed in `doc`'s `owner` field is equal to the passed
in actor. For example if the actor was `"steve"` and the document was 
`{owner: "steve", content: "hello world"}` then `"steve"` would be allowed
to `"get"` that document.

Data filtering means asking a slightly different question. Instead of “Can
this actor do this action on this specific resource?”, we want to ask “What
are all the resources that this actor can do this action on?”.

One way to answer this question is to take every Document in the system and
call `is_allowed` on it. This isn’t efficient and many times is just
impossible. There could be thousands of Documents in a database but only 3
that have the owner `"steve"`. Instead of fetching every document and passing
it into Oso, it's better to ask the database for only the documents that
have the owner `"steve"`. This process of filtering the data in your data
store, based on the logic in your policy is what we call “Data Filtering”.

{{% ifLang "python" %}}
{{% callout "ORM Integrations" "blue" %}}
If you are using one of our ORM adapter libraries like `sqlalchemy-oso` or
`django-oso` data filtering is already built in and you won't have to worry
about integrating it yourself. See docs for the ORM library instead.
{{% /callout %}}
{{% /ifLang %}}

Data filtering is about query level enforcement. Oso will take the logic in
the policy and turn it into a query for the authorized data. Examples could
include an ORM filter object, an HTTP request or an elastic-search query.
The query object and the way the logic maps to a query are both user defined.

Data filtering is initiated through two methods on oso.

`{{% exampleGet "authorizedResources" %}}` returns a list of all the
resources a user is allowed to do an action on. The results of a built and
executed query.

`{{% exampleGet "authorizedQuery" %}}` returns the query object itself.
This lets you add additional filters or sorts or any other data to it before
executing it.

You must define how to build queries and a few other details when you register classes to enable these methods.

# Implementing data filtering

## Query Functions

There are three Query functions that must be implemented. These define what a query is for your application, how the logic in the policy maps to them, how to execute them and how to combine two queries.

### Build a Query

`{{% exampleGet "buildQuery" %}}` takes a list of `Filter`s and returns a
`Query`

`Filter`s are individual pieces of logic that must apply to the data being
fetched.

Filters have a `kind`, a `field` and a `value`. Their meaning depends on the
`kind` field.

- `Eq` means that the field must be equal to the value.
- `Neq` means that the field must not be equal to the value.
- `In` means that the field must be equal to one of the values in value.
Value will be a list.
- `Contains` means that the field must contain the value. This only applies
if the field is a list.

### Execute a Query

`{{% exampleGet "execQuery" %}}` takes a query and returns a list of the results.

### Combine Queries

`{{% exampleGet "combineQuery" %}}` takes two queries and returns a new
query that returns the union of the other two. For example if the two
queries are SQL queries combine could `UNION` them. If they were HTTP
requests `{{% exampleGet "combineQuery" %}}` could put them in an array and 
`` could handle executing an array of queries
and combining the results.

You can define functions that apply to all types with
`{{% exampleGet "setDataFilteringQueryDefaults" %}}`. Or you can pass type specific ones when you register a class.

## Types

The other thing you have to define to use data filtering are types for
registered classes. These let oso know what the type of the fields are. Oso
needs this information to handle specializers and other things in the policy
when we don't have a concrete resource. The types are a 
{{% exampleGet "map" %}} from field name to type.

# Example
{{< literalInclude
      dynPath="exampleAPath"
      from="docs: begin-a1"
      to="docs: end-a1"
      fallback="no" >}}

For each class we need to register it and define the query functions.

{{< literalInclude
      dynPath="exampleAPath"
      from="docs: begin-a2"
      to="docs: end-a2"
      fallback="no" >}}

Then we can load a policy and query it.

{{< literalInclude
      dynPath="policyAPath"
      fallback="no" >}}

{{< literalInclude
      dynPath="exampleAPath"
      from="docs: begin-a3"
      to="docs: end-a3"
      fallback="no" >}}

# Relations

Often times you need data that is not contained on the object to make
authorization decisions. This comes up when the role required to
do something is implied by a role on it's parent object. For instance,
you want to check the organization for a repository but that data isn't
embedded on the repository object. You can add a `Relation` type to the type
definition that states how the other resource is related to this one. Then
you can access this field in the policy like any other field and it will
fetch the data when it needs it (via the query functions).

`Relation`s are a special type that tells oso how one Class is related to another. They specify what the related type is and how it's related.

- `kind` is either "one" or "many". "one" means there is one related object and "many" means there is a list of related objects.
- `other_type` is the class of the related objects.
- `my_field` Is the field on this object that matches `other_field`.
- `other_field` Is the field on the other object that matches `this_field`.

The my_field / other_field relationship is similar to a foreign key. It lets oso know what fields to match up with building a query for the other type.

# Example

{{< literalInclude
      dynPath="exampleBPath"
      from="docs: begin-b1"
      to="docs: end-b1"
      fallback="no" >}}

We now have two sets of query functions. Our `{{% exampleGet "buildQuery" %}}`
function depends on the class but our `{{% exampleGet "execQuery" %}}` and
`{{% exampleGet "combineQuery" %}}` functions are the same for all types so we
can set them with `{{% exampleGet "setDataFilteringQueryDefaults" %}}`.

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

# Evaluation
When oso is evaluating data filtering methods it uses queries to fetch objects. If there are multiple types involved it will make multiple queries and substitute in the results when needed.
In the above example we are fetching Repositories, but we are basing our fetch on some information about their related Organization. To resolve the query Oso first fetches the relevent Organizations (based in this case on role assignments), and then uses the `Relation` definition to substitute in their ids to the query for Repositories. This is the main reason to use `Relation`s, they let oso know how different classes are related so we can resolve data filtering queries.
Relation fields also work when you are not using data filtering methods and are just using `authorize` or another method where you have an object to pass in. In that case the query functions are still called to get related objects so if you're using a `Relation` to a type, you must define query functions for that type.

# Limitations

There are a few limitations to what you can do while using data filtering. You can not call any methods on the passed in resource and you can not pass the resource as an argument to any methods. Many cases where you would want to do this are better handled by Relation fields.

Some Polar expressions are not supported. `not`, `cut` and `forall` are not allowed in policies that want to use data filtering. Numberic comparisons with the `<` `>` `<=` and `>=` are not currently supported either.

Relations only support matching on a single field.