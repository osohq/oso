---
title: Application types
weight: 2
---
<!-- 
TODO: convert into language-specific guides -->

<!-- JAVA EXAMPLES -->
# Application Types

Any type defined in an application can be passed into oso, and its
attributes may be accessed from within a policy. Using application types
make it possible to take advantage of an app’s existing domain model. For example:

Python

```
allow(actor, action, resource) if actor.is_admin;
```

The above rule expects the `actor` variable to be a Python instance with the attribute `is_admin`.
The Python instance is passed into oso with a call to `is_allowed()`:

```
class User:
    def __init__(self, name, is_admin):
        self.name = name
        self.is_admin = is_admin

user = User("alice", True)
assert(oso.is_allowed(user, "foo", "bar"))
```

The code above provides a `User` object as the *actor* for our `allow` rule. Since `User` has an attribute
called `is_admin`, it is evaluated by the policy and found to be true.

Ruby

```
allow(actor, action, resource) if actor.is_admin;
```

The above rule expects the `actor` variable to be a Ruby instance with the attribute `is_admin`.
The Ruby instance is passed into oso with a call to `Oso#allowed?`:

```
class User
  attr_reader :name
  attr_reader :is_admin

  def initialize(name, is_admin:)
    @name = name
    @is_admin = is_admin
  end
end

user = User.new("alice", is_admin: true)
raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
```

The code above provides a `User` object as the *actor* for our `allow` rule. Since `User` has an attribute
called `is_admin`, it is evaluated by the policy and found to be true.

Java

```
allow(actor, action, resource) if actor.isAdmin;
```

The above rule expects the `actor` variable to be a Java instance with the field `isAdmin`.
The Java instance is passed into oso with a call to `Oso.isAllowed`:

```
public class User {
    public boolean isAdmin;
    public String name;

    public User(String name, boolean isAdmin) {
        this.isAdmin = isAdmin;
        this.name = name;
    }

    public static void main(String[] args) {
        User user = new User("alice", true);
        assert oso.isAllowed(user, "foo", "bar");
    }
}
```

The code above provides a `User` object as the *actor* for our `allow` rule. Since `User` has a field
called `isAdmin`, it is evaluated by the Polar rule and found to be true.

Node.js

```
allow(actor, action, resource) if actor.isAdmin;
```

The above rule expects the `actor` variable to be a JavaScript object
with an `isAdmin` field. The JavaScript object is passed into oso
with a call to `Oso.isAllowed`:

```
class User {
  constructor (name, isAdmin) {
    this.name = name;
    this.isAdmin = isAdmin;
  }
}

const user = new User("alice", true);

(async () => {
  const decision = await oso.isAllowed(user, 'foo', 'bar');
  assert(decision);
})();
```

The code above provides a `User` instance as the *actor* for our
`allow` rule. Since `User` has a field called `isAdmin`, it is
evaluated by the Polar rule and found to be true.

Rust

```
allow(actor, action, resource) if actor.is_admin;
```

The above rule expects the `actor` variable to be a Rust instance with the attribute `is_admin`.
The Rust instance is passed into oso with a call to `oso.is_allowed`:

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    name: String,
    #[polar(attribute)]
    is_admin: bool,
}
oso.register_class(User::get_polar_class())?;
let user = User { name: "alice".to_string(), is_admin: true };
assert!(oso.is_allowed(user, "foo", "bar")?);
```

The code above provides a `User` object as the *actor* for our `allow` rule. Since `User` has an attribute
called `is_admin`, it is evaluated by the policy and found to be true.

In addition to accessing attributes, you can also call methods on application
instances in a policy:

```
allow(actor, action, resource) if actor.isAdminOf(resource);
```

If the method takes arguments, they must currently be supplied as
positional arguments, even if the method is defined to take keyword
arguments.

## Registering Application Types

Instances of application types can be constructed from inside an oso policy
using the New operator if the class has been **registered**:

Python

or the `polar_class()` decorator:

```
oso.register_class(User)
```

Once the class is registered, we can make a `User` object in Polar.
This can be helpful for writing inline test queries:

```
?= allow(new User(name: "alice", is_admin: true), "foo", "bar");
```

Initialization arguments provided in this way are passed as keywords.
We can also pass positional arguments to the class constructor:

```
?= allow(new User("alice", true), "foo", "bar");
```

Ruby

```
OSO.register_class(User)
```

Once the class is registered, we can make a `User` object in Polar.
This can be helpful for writing inline test queries:

```
?= allow(new User("alice", is_admin: true), "foo", "bar");
```

Java

```
public static void main(String[] args) {
    oso.registerClass(User.class);
}
```

You may register a Java class with a particular [Constructor](https://docs.oracle.com/javase/10/docs/api/java/lang/reflect/Constructor.html),
but the default behavior is to choose one at instantiation time
based on the classes of the supplied arguments. For the example
above, this would probably be a constructor with a signature like
`public User(String name, bool isAdmin)`.
See Java Authorization Library for more details.

Once the class is registered, we can make a `User` object in Polar.
This can be helpful for writing inline test queries:

```
?= allow(new User("alice", true), "foo", "bar");
```

We must pass positional arguments to the class constructor because
Java does not support keyword arguments.

Node.js

```
oso.registerClass(User);
```

Once the class is registered, we can make a `User` object in Polar.
This can be helpful for writing inline test queries:

```
?= allow(new User("alice", true), "foo", "bar");
```

We must pass positional arguments to the class constructor because
JavaScript does not support keyword arguments.

Rust

`register_class` takes as input a `Class`, which can be constructed
either using the `#[derive(PolarClass)]` proc-macro, or manually using
`Class::new::<T>()`:

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    name: String,
    #[polar(attribute)]
    is_admin: bool,
}

impl User {
    fn new(name: String, is_admin: bool) -> Self {
        Self { name, is_admin }
    }

    fn is_called_alice(&self) -> bool {
        self.name == "alice"
    }
}

oso.register_class(
   User::get_polar_class_builder()
        .set_constructor(User::new)
        .add_method("is_called_alice", User::is_called_alice)
        .build(),
)?;
```

Once the class is registered, we can make a `User` object in Polar.
This can be helpful for writing inline test queries:

```
?= allow(new User("bob", true), "foo", "bar");
?= new User("alice", true).is_called_alice();
```

The Rust library only supports calling constructors and methods with positional
arguments, since Rust itself does not have keyword arguments.

Registering classes also makes it possible to use Specialization
and the Matches Operator with the registered class. Here’s what
specialization on an application type looks like.

In our previous example, the **allow** rule expected the actor to be a `User`,
but we couldn’t actually check that type assumption in the policy. If we register
the `User` class, we can write the following rule:

```
allow(actor: User, action, resource) if actor.name = "alice";
```

This rule will only be evaluated when the actor is a `User`; the
`actor` argument is *specialized* on that type. We could also use
`matches` to express the same logic on an unspecialized rule:

```
allow(actor, action, resource) if actor matches User{name: "alice"};
```

Either way, using the rule could look like this:

Python

```
oso.register_class(User)

user = User("alice", True)
assert oso.is_allowed(user, "foo", "bar")
assert not oso.is_allowed("notauser", "foo", "bar")
```

Ruby

```
OSO.register_class(User)
user = User.new("alice", is_admin: true)
raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
raise "should not be allowed" unless not OSO.allowed?(actor: user, action: "foo", resource: "bar")
```

Java

```
public static void main(String[] args) {
    oso.registerClass(User.class);

    User user = new User("alice", true);
    assert oso.isAllowed(user, "foo", "bar");
    assert !oso.isAllowed("notauser", "foo", "bar");
}
```

Node.js

```
oso.registerClass(User);
const user = new User('alice', true);

(async () => {
  assert.equal(true, await oso.isAllowed(user, "foo", "bar"));
  assert.equal(false, await oso.isAllowed("notauser", "foo", "bar"));
})();
```

Rust

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    name: String,
    #[polar(attribute)]
    is_admin: bool,
}
oso.register_class(User::get_polar_class())?;

let user = User { name: "alice".to_string(), is_admin: true };
assert!(oso.is_allowed(user, "foo", "bar")?);
assert!(!oso.is_allowed("notauser", "foo", "bar")?);
```

**NOTE**: Type specializers automatically respect the
**inheritance** hierarchy of our application classes. See our Resources with Inheritance guide for an
in-depth example of how this works.

Once a class is registered, class or static methods can also be called from oso policies:

Python

```
allow(actor: User, action, resource) if actor.name in User.superusers();
```

```
class User:
    ...
    @classmethod
    def superusers(cls):
        """ Class method to return list of superusers. """
        return ["alice", "bhavik", "clarice"]

oso.register_class(User)

user = User("alice", True)
assert(oso.is_allowed(user, "foo", "bar))
```

Ruby

```
allow(actor: User, action, resource) if actor.name in User.superusers();
```

```
class User
  # ...
  def self.superusers
    ["alice", "bhavik", "clarice"]
  end
end

OSO.register_class(User)

user = User.new("alice", is_admin: true)
raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
```

Java

```
allow(actor: User, action, resource) if actor.name in User.superusers();
```

```
public static List<String> superusers() {
    return List.of("alice", "bhavik", "clarice");
}

public static void main(String[] args) {
    oso.registerClass(User.class);

    User user = new User("alice", true);
    assert oso.isAllowed(user, "foo", "bar");
}
```

Node.js

```
allow(actor: User, action, resource) if actor.name in User.superusers();
```

```
class User {
  constructor (name, isAdmin) {
    this.name = name;
    this.isAdmin = isAdmin;
  }

  static superusers() {
    return ['alice', 'bhavik', 'clarice'];
  }
}

oso.registerClass(User);
const user = new User('alice', true);

(async () => assert(await oso.isAllowed(user, "foo", "bar")))();
```

Rust

```
allow(actor: User, action, resource) if actor.name in User.superusers();
```

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    name: String,
}

impl User {
    fn superusers() -> Vec<String> {
        vec![
            "alice".to_string(),
            "bhavik".to_string(),
            "clarice".to_string(),
        ]
    }
}

oso.register_class(
    User::get_polar_class_builder()
        .add_class_method("superusers". User::superusers)
        .build(),
)?;

let user = User { name: "alice".to_string() };
assert!(oso.is_allowed(user, "foo", "bar)?);
```

## Built-in Types

Methods called on the Polar built-in types `String`, `Dictionary`, `Number`,
and `List` punt to methods on the corresponding application language class.
That way you can use familiar methods like `str.startswith()` on strings
regardless of whether they originated in your application or as a literal in
your policy. This applies to all of the Polar supported types,
in any supported application language. For examples using built-in types,
see the Libraries guides.

**WARNING**: Do not attempt to mutate a literal using a method on it.
Literals in Polar are constant, and any changes made to such objects
by calling a method will not be persisted.

### `nil`

In addition to the built-in types, Polar registers a constant named
`nil` that maps to an application-language-specific “null” value:

| Polar

 | Python

 | Ruby

 | Java

 | JavaScript

 | Rust

 | SQL

 |     |
 | --- ||  |  |  |  |  |
 | `nil` |

   | `None`

   | `nil`

  | `null`

 | `null`

       | `Option::<T>::None`

 | `NULL`

 |
The Polar value `nil` is not equal to either the empty list `[]`
or the boolean value `false`. It is intended to be used with application
methods that return a null value.

## Summary


* **Application types** and their associated application data are available
within policies.


* Types can be **registered** with oso, in order to:

    
        * Create instances of application types in policies


        * Leverage the inheritance structure of application types with **specialized
    rules**, supporting more sophisticated access control models.


* You can use **built-in methods** on primitive types & literals like strings
and dictionaries, exactly as if they were application types.
