---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/inheritance
images: {}
path: /using-examples-inheritance
title: Share Rules Across Resources
weight: 10
description: |
  Oso policies make it possible to share rules across related resource types, and override them as needed.
aliases:
  - ../../../using/examples/inheritance.html
---

# Share Rules Across Resources

Some applications have many resources that should have similar authorization
rules applied to them. This is a common scenario in workflow driven
applications that have different user types and a large number of resources.

A common set of rules will apply to many resources, with some exceptions. In
this guide, we will cover various ways for modeling this scenario with Oso.

## Setup

In this example, we will consider a hypothetical EMR (electronic medical
records) application. We’ll discuss a few resources:

* **Order**: A record of an action that medical staff will perform on a patient
* **Test**: A diagnostic test that will be performed on a patient.
* **Lab**: A lab test that will be performed on a patient.

These resources are all examples of different types of patient data.

## Basic Policy

Let’s start by considering a basic policy controlling access to these three
resources:

{{< literalInclude path="examples/inheritance/01-polar.polar" >}}

Let’s take a look at the first rule in the policy. This `allow` rule permits an
actor to perform the `"read"` action on an `Order` if:

1. The actor’s `role` property is equal to `"medical_staff"`.
2. The actor has treated the patient associated with the `Order` in question,
   which is verified by calling the actor's `treated()` method.

Note the head of the rule. Each argument uses a type specializer to ensure this
rule only applies to certain types of resources and actors. This rule indicates
that the `actor` argument must be an instance of the `User` class and the
`resource` argument must be an instance of the `Order` class.

This policy meets our goal above. We have expressed the same rule for the three
types of patient data, but it is a bit repetitive. Let’s try to improve it.

## Using a Rule to Express Common Behavior

Our policy doesn’t just need to contain `allow` rules. We can write any rules
we’d like and compose them as needed to express our policy!

{{< literalInclude path="examples/inheritance/02-nested-rule.polar" >}}

Now, we’ve taken the repeated logic and expressed it as the
`can_read_patient_data` rule. When the `allow` rule is evaluated, Oso will
check if the `can_read_patient_data` is satisfied. The policy is much shorter!

Unfortunately, we’ve lost one property of our last policy: the specializers.
This rule would be evaluated for any type of resource — not just our three
examples of patient data above. That’s not what we want.

## Bringing Back Specializers

We can combine this idea with our first policy to make sure only our three
patient data resources use the `can_read_patient_data` rule.

{{< literalInclude path="examples/inheritance/03-specializer.polar"
                   from="START MARKER" >}}

Now, we still have three rules, but the body isn’t repeated anymore.

## One Rule to Rule Them All

We haven’t talked about the application side of this yet. So far, we’ve assumed
`Order`, `Lab`, and `Test` are application classes.

Here’s how they might be implemented:

{{< literalInclude dynPath="classesPath"
                   from="start-patient-data"
                   to="end-patient-data" >}}

We used inheritance to capture some of the common functionality needed (storing
the patient). In a real application these would probably be ORM models.

We can use the same idea to shorten our policy even further!

{{< literalInclude path="examples/inheritance/04-one-specializer.polar" >}}

Now, this `allow` rule will be evaluated for any instance that is a subclass of
`PatientData`. Polar understands the class inheritance structure when selecting
rules to evaluate!

<!-- TODO: include when groups are back
Working with groups
-------------------

This worked well for us, but remember this is just an example. Not all
applications may have encoded relationships this way. Maybe when we wrote our
code we didn't create a ``PatientData`` class, and just implemented ``Lab``,
``Order`` and ``Test`` separately. We still want to treat them as one concept
in our policy, but don't want to change our application.

Polar includes a ``group`` construct that can be used for exactly this purpose:

.. literalinclude:: /examples/inheritance/05-group.polar
   :language: polar
   :emphasize-lines: 1

The :ref:`group` declaration creates a new type in our Polar file called
``PatientData``. This time, we tell Polar that ``Lab``, ``Order`` and ``Test``
are part of this group. We can write our rule in the same way as before. -->

## Summary

In this guide, we saw an example of an application policy that could result in
significant repetition. We tried out a few strategies for representing common
policy across many resource types. First, we wrote a custom rule that moved
duplicated logic into one place. Then we used specializers and application
types to condense our policy even further.

<!-- TODO: include when groups are back
Finally, we saw how groups & inheritance can both be exploited to
write flexible policies that accurately model our application's domain model. -->
