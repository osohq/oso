---
title: "Data Filtering Preview"
weight: 1
showContentForAnyLanguage: true
---

# Data Filtering Preview

## What is data filtering
When you evaluate an oso policy (using `is_allowed`) for a specific `actor`, `action` and `resource`, oso evaluates the allow rule(s) you have defined to determine if that `actor` is allowed to do that `action` on that `resource`. For instance if you have a policy that says something like this,
```
allow(actor, "get", doc: Document) if doc.owner = actor;
```
Then oso checks that the passed in `doc`'s owner field is equal to the passed in actor. Eg. if the actor is `"steve"` and the document is `{owner: "steve", content: "..."}` Then steve would be allowed to `"get"` that document.

Data filtering is asking a slightly different question of the policy. Instead of asking "Can this actor do this action on this specific resource?", we want to ask "What are all the resources that this actor can do this specific action on?".
One way to answer this question would be to take every Document in the system and call `is_allowed` on it. This isn't efficient and many times is just impossible. There could be thousands of Documents in a database but only 3 that have the owner "steve". Instead of fetching every document and passing it into oso, we would like to ask our database for only the documents that have the owner "steve". This process of filtering the data in our data store, based on the logic in our policy is what we call "Data Filtering".

## Callout that if you're using sqlalchemy or django data filtering is built in and you don't have to worry about this
link to the docs instead.

## How data filtering works
Data filtering works by evaluating the policy without passing in a resource. Instead of checking things on the resource
like `resource.name = "steve"`, we collect up what each of those individual checks would be. These are called `Constraint`s. We then call a function the user has registered with those constraints so that the user can query their own data store with them.

## How to use data filtering
To use data filtering you need to provide two additional things when you register your classes.

### Types
Oso needs to know the types of all the fields in your class. This is how we know what `resource.name` will be when we don't have a concrete resource to check the field on. This lets polar code work the same way it does normally and things like specializers match.
There is one special type called `Relationship` that tells polar the field refers to a related object. This lets you reference a related object in polar and tells us how the current object is related to the other object.

### Fetchers
The other thing oso has to know to use data filtering are how to fetch data. These are functions that take as input a list of `Constraint` objects. The function is then responsible for selecting data that matches all of the constraint from the database, or an api, or wheverver the data lives. This is the place that data filtering integrates with your data store. 

### Using
You can pass both of these things as arguments when registering a class and then you can use data filtering. Here's  an example.

## Example



## Limitations
Currently there are some limitations to what you can do while using data filtering. You can not call any methods on the passed in resource (or any of it's properties). You also can not pass the resource as an argument to a method. Many cases where you would want to do this are better handled by Relationship fields.

Some polar expressions are not supported but may be in the future. `not`, `cut` and `forall` are not allowed in policies that want to use data filtering. Numeric comparisons are also not yet supported. `< > <= >= !=`

For now, Relationships only support matching on a single field. 

{{% ifLang "python" %}}
## Python

python

{{% /ifLang %}}

{{% ifLang "ruby" %}}
## Ruby

ruby

{{% /ifLang %}}

{{% ifLang "node" %}}
## Javascript

node

{{% /ifLang %}}

{{< ifLangExists >}}
{{% ifLang not="node" %}}
{{% ifLang not="python" %}}
{{% ifLang not="ruby" %}}

Data filtering is coming soon for {{< lang >}}!

If you want to get data filtering in your app now or just want to
register your interest for Data Filtering in {{< lang >}} [drop into our Slack](http://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Roles%20support%20for%20{{< currentLanguage >}}&body=I%27m%20interested%20in%20Oso%20roles%20support%20for%20{{< currentLanguage >}}">send an email</a>
to our engineering team and we'll unblock you.
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLangExists %}}
