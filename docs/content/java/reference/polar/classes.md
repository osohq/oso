---
title: Java Types in Polar
weight: 2
aliases:
  - /using/libraries/java/index.html
description: |
    Reference for using Java types in Polar.
---

## Working with Java Types

Oso’s Java authorization library lets you write policy rules over Java objects
directly. This document explains how different types of Java objects can be
used in Oso policies.

{{% callout "Note" "blue" %}}
More detailed examples of working with application classes can be found in our
[Guides](guides).
{{% /callout %}}

### Class Instances

You may pass an instance of any Java class into Oso and access its methods and
fields from your policy (see [Application
Types](guides/policies#instances-and-fields)).

Java instances can be constructed from within an Oso policy using the
[`new`](polar-syntax#new) operator:

```polar
new User("alice@example.com")
```

To construct instances of a Java class, the class must be **registered** using
the `registerClass()` method:

```java
oso.registerClass(User.class)
```

If you want to refer to the class using another name from within a policy, you
may supply an alias:

```java
oso.registerClass(Person.class, "User")
```

At instantiation time, Oso will search the list returned by
[`Class.getConstructors()`](<https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/lang/Class.html#getConstructors()>)
for a constructor that is applicable to the supplied positional constructor
arguments. For example, given the Polar expression `new User("alice@example.com")`, Oso will search for a `Constructor` with one
parameter compatible with `String.class`, e.g.:

```java
public User(String username) { ... }
```

Applicability is determined using [`Class.isAssignableFrom(Class<?>
cls)`](<https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/lang/Class.html#isAssignableFrom(java.lang.Class)>),
which allows arguments that are instances of subclasses or implementations of
interfaces to properly match the constructor’s parameter types.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)).

{{% callout "Note" "blue" %}}
Java primitives may be passed into Oso, but numbers and booleans created in
an Oso policy will be converted to
[autoboxed](https://docs.oracle.com/javase/tutorial/java/data/autoboxing.html)
Integer, Float, and Boolean types respectively.

This means that methods called from Oso must have autoboxed argument types.
E.g.:

```java
class Foo {
    public static unboxed(int a, int b) {
        // ...
    }
    public static boxed(Integer a, Integer b) {
        // ...
    }
}
```

The `boxed()` method may be called from a policy, but attempting to call
`unboxed()` will fail.
{{% /callout %}}

### Strings

Java Strings are mapped to Polar [strings](polar-syntax#strings). Java’s String
methods may be accessed from policies:

```polar
allow(actor, _action, _resource) if actor.username.endsWith("example.com");
```

```java
public class User {
    public String username;

    public User(String username) {
        this.username = username;
    }

    public static void main(String[] args) {
        User user = new User("alice@example.com");
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

### Lists and Arrays

Java
[Arrays](https://docs.oracle.com/javase/tutorial/java/nutsandbolts/arrays.html)
_and_ objects that implement the
[List](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/util/List.html)
interface are mapped to Polar [lists](polar-syntax#lists). Java’s `List`
methods may be accessed from policies:

```polar
allow(actor, _action, _resource) if actor.groups.contains("HR");
```

```java
public class User {
    public List<String> groups;

    public User(List<String> groups) {
        this.groups = groups;
    }

    public static void main(String[] args) {
        User user = new User(List.of("HR", "payroll"));
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

Note that the `isAllowed()` call would also succeed if `groups` were an Array.

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate lists in place. E.g., `add()` will
have no effect on a list in Polar.
{{% /callout %}}

Likewise, lists constructed in Polar may be passed into Java methods:

```polar
allow(actor, _action, _resource) if actor.has_groups(["HR", "payroll"]);
```

```java
public class User {
    ...

    public boolean hasGroups(List<String> groups) {
        for(String g : groups) {
            if (!this.groups.contains(g))
                return false;
        }
        return true;
    }

    public static void main(String[] args) {
        User user = new User(List.of("HR", "payroll"));
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

Java methods like
[`List.get`](<https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/util/List.html#get(int)>)
may be used for random access to list elements, but there is currently no Polar
syntax for that is equivalent to the Java expression `user.groups[1]`. To
access the elements of a list without using a method, you may iterate over it
with [the `in` operator](polar-syntax#in-list-membership) or destructure it
with [pattern matching](polar-syntax#patterns-and-matching).

### Maps

Java objects that implement the
[Map](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/util/Map.html) interface
are mapped to Polar [dictionaries](polar-syntax#dictionaries):

```polar
allow(actor, _action, _resource) if actor.roles.project1 = "admin";
```

```java
public class User {
    public Map<String, String> roles;

    public User(Map<String, String> roles) {
        this.roles = roles;
    }

    public static void main(String[] args) {
        User user = new User(Map.of("project1", "admin"));
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

Likewise, dictionaries constructed in Polar may be passed into Java methods.

### Enumerations

You may iterate over a Java
[Enumeration](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/util/Enumeration.html)
(or anything that can be converted to one, such as a `Collection` or
`Iterable`) using Polar's [`in` operator](polar-syntax#in-list-membership):

```polar
allow(actor, _action, _resource) if "payroll" in actor.getGroups();
```

```java
public class User {
    public List<String> getGroups() {
        return List.of("HR", "payroll");
    }

    public static void main(String[] args) {
        User user = new User(Map.of("project1", "admin"));
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

### `null`

The Java `null` reference is registered as the Polar constant
[nil](reference/polar/polar-syntax#nil). If a Java method can return
`null`, you may want to compare the result to `nil`:

```polar
allow(actor, _action, _resource) if actor.getOptional() != nil;
```

```java
public class User {
    ...

    public Thing getOptional() {
        if someCondition() {
            return new Thing();
        } else {
            return null;
        }
    }
}
```

### Java → Polar Types Summary

| Java type           | Polar type   |
| ------------------- | ------------ |
| `int`/`Integer`     | `Integer`    |
| `float`/`Float`     | `Float`      |
| `double`/`Double`   | `Float`      |
| `boolean`/`Boolean` | `Boolean`    |
| `List`              | `List`       |
| `Array`             | `List`       |
| `Map`               | `Dictionary` |
| `String`            | `String`     |
