---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/user_types
images: {}
path: /using-examples-user-types
title: Multiple Actor Types
---

# Multiple Actor Types

Recall that in oso, Actors represent request-makers, the “who” of an authorization request.
Actors are commonly human users, but might also be machines, servers, or other applications.
Many applications support multiple types of Actors, and often different Actor types require different
authorization logic.

In this guide, we’ll walk through a policy for an application with two Actor types: **Customers** and
**Internal Users**.

**NOTE**: This guide assumes you are familiar with oso’s Glossary.

## A Tale of Two Actors

Our example application has customers and internal users. Customers are allowed to access the customer dashboard,
and internal users are allowed to access the customer dashboard as well as an internal dashboard. We can write a simple
policy to express this logic.

Python

Let’s start by defining Python classes to represent customers and internal users:

```
from oso import polar_class


@polar_class
class Customer:
    def __init__(self, id):
        self.id = id


@polar_class
class InternalUser:
    def __init__(self, id):
        self.id = id


```

Ruby

Let’s start by defining Ruby classes to represent customers and internal users:

```
require 'oso'

OSO ||= Oso.new

class Customer
  def initialize(id)
    @id = id
  end
end

OSO.register_class(Customer)

class InternalUser
  def initialize(id)
    @id = id
  end
end

OSO.register_class(InternalUser)
```

Java

Java example coming soon.

Node.js

Let’s start by defining JavaScript classes to represent customers and
internal users:

```
const { Oso } = require('oso');

const oso = new Oso();

class Customer {
  constructor(id) {
    this.id = id;
  }
}

oso.registerClass(Customer);

class InternalUser {
  constructor(id) {
    this.id = id;
  }
}

oso.registerClass(InternalUser);
```

We can now write a simple policy over these Actor types:

Python

```
# Internal users have access to both the
# internal and customer dashboards
allow(actor: InternalUser, "view", "internal_dashboard");
allow(actor: InternalUser, "view", "customer_dashboard");

# Customers only have access to the customer dashboard
allow(actor: Customer, "view", "customer_dashboard");
```

Ruby

```
# Internal users have access to both the
# internal and customer dashboards
allow(actor: InternalUser, "view", "internal_dashboard");
allow(actor: InternalUser, "view", "customer_dashboard");

# Customers only have access to the customer dashboard
allow(actor: Customer, "view", "customer_dashboard");
```

Java

Java example coming soon.

Node.js

```
# Internal users have access to both the
# internal and customer dashboards
allow(_actor: InternalUser, "view", "internal_dashboard");
allow(_actor: InternalUser, "view", "customer_dashboard");

# Customers only have access to the customer dashboard
allow(_actor: Customer, "view", "customer_dashboard");
```

This policy uses specialized rules to control rules execution based on
the Actor types that is passed into the authorization request.

To finish securing our dashboards, we need to **enforce** our policy by
adding authorization requests to our application.
Where and how authorization requests are used is up to the application developer.

For our example, making a request might look like this:

Python

```
def customer_dashboard_handler(request):
    oso = get_oso()
    actor = user_from_id(request.id)
    allowed = oso.is_allowed(actor=actor, action="view", resource="customer_dashboard")


def user_from_id(id):
    user_type = db.query("SELECT type FROM users WHERE id = ?", id)
    if user_type == "internal":
        return InternalUser(id)
    elif user_type == "customer":
        return Customer(id)


```

Ruby

```
def customer_dashboard_handler(request)
  actor = user_from_id(request.id)
  OSO.allowed?(actor: actor, action: 'view', resource: 'customer_dashboard')
end

def user_from_id(id)
  case db.query('SELECT type FROM users WHERE id = ?', id)
  when 'internal'
    InternalUser.new(id: id)
  when 'customer'
    Customer.new(id: id)
  end
end
```

Java

Java example coming soon.

Node.js

```
async function customerDashboardHandler(request) {
  const actor = userFromId(request.id);
  return oso.isAllowed(actor, 'view', 'customer_dashboard');
}

function userFromId(id) {
  const userType = db.query('SELECT type FROM users WHERE id = ?', id);
  if (userType === 'internal') {
    return new InternalUser(id);
  } else if (userType === 'customer') {
    return new Customer(id);
  }
}
```

Hooray, our customer and internal dashboards are now secure!

## Adding Actor Attributes

Since we saved so much time on authorization, we’ve decided to add another dashboard to our application,
an **accounts dashboard**. The accounts dashboard should only be accessed by **account managers** (a type of internal user).
Since we’re experts at securing dashboards, we should be able to add this authorization logic to our policy in no time.
A simple way to solve this problem is with RBAC.

We can add a `role` attribute to our `InternalUser` class:

Python

```
@polar_class
class InternalUser:
    def role(self):
        yield db.query("SELECT role FROM internal_roles WHERE id = ?", self.id)


```

Ruby

```
class InternalUser
  attr_reader :id

  def initialize(id)
    @id = id
  end

  def role
    db.query('SELECT role FROM internal_roles WHERE id = ?', id)
  end
end

OSO.register_class(InternalUser)
```

Java

Java example coming soon.

Node.js

```
class InternalUser {
  constructor(id) {
    this.id = id;
  }

  role() {
    return db.query('SELECT role FROM internal_roles WHERE id = ?', this.id);
  }
}

oso.registerClass(InternalUser);
```

Then add the following rule to our policy:

Python

```
# Internal users can access the accounts dashboard if
# they are an account manager
allow(actor: InternalUser, "view", "accounts_dashboard") if
    actor.role() = "account_manager";
```

Ruby

```
# Internal users can access the accounts dashboard if
# they are an account manager
allow(actor: InternalUser, "view", "accounts_dashboard") if
    actor.role = "account_manager";
```

Java

Java example coming soon.

Node.js

```
# Internal users can access the accounts dashboard if
# they are an account manager
allow(actor: InternalUser, "view", "accounts_dashboard") if
    actor.role() = "account_manager";
```

This example shows a clear benefit of using different classes to represent different Actor types: the ability
to add custom attributes. We can add attributes specific to internal users, like roles, to the `InternalUser` class
without adding them to all application users.

We’ve been able to secure the accounts dashboard with a few lines of code, but we’re not done yet!

Account managers are also allowed to access **account data**, but only for accounts that they manage.
In order to implement this logic, we need to know the accounts of each account manager.

This is a compelling case for creating a new Actor type for account managers that has its own
attributes:

Python

```
@polar_class
class AccountManager(InternalUser):
    def customer_accounts(self):
        yield db.query("SELECT id FROM customer_accounts WHERE manager_id = ?", self.id)


```

Ruby

```
class AccountManager < InternalUser
  def customer_accounts
    db.query('SELECT id FROM customer_accounts WHERE manager_id = ?', id)
  end
end
```

Java

Java example coming soon.

Node.js

```
class AccountManager extends InternalUser {
  customerAccounts() {
    return db.query(
      'SELECT id FROM customer_accounts WHERE manager_id = ?',
      this.id
    );
  }
}
```

Since account managers are also internal users, we’ve made the `AccountManager` type extend `InternalUser`.
This means that our rules that specialize on `InternalUser` will still execute for account managers (see Resources with Inheritance).

Let’s add the following lines to our policy:

Python

```
# Account managers can access the accounts dashboard
allow(actor: AccountManager, "view", "accounts_dashboard");

# Account managers can access account data for the accounts
# that they manage
allow(actor: AccountManager, "view", resource: AccountData) if
    resource.account_id in actor.customer_accounts();
```

Ruby

```
# Account managers can access the accounts dashboard
allow(actor: AccountManager, "view", "accounts_dashboard");

# Account managers can access account data for the accounts
# that they manage
allow(actor: AccountManager, "view", resource: AccountData) if
    resource.account_id in actor.customer_accounts;
```

Java

Java example coming soon.

Node.js

```
# Account managers can access the accounts dashboard
allow(_actor: AccountManager, "view", "accounts_dashboard");

# Account managers can access account data for the accounts
# that they manage
allow(actor: AccountManager, "view", resource: AccountData) if
    resource.accountId in actor.customerAccounts();
```

The first rule replaces the RBAC rule we previously used to control access to
the accounts dashboard. The second rule controls access to account data.

Python

For the purposes of this example, let’s assume that `AccountData` is a
resource that has an `account_id` attribute.

We can update our application code slightly to generate `AccountManager` users:

```
def user_from_id(id):
    user_type = db.query("SELECT type FROM users WHERE id = ?", request.id)
    if user_type == "internal":
        actor = InternalUser(request.id)
        if actor.role() == "account_manager":
            return AccountManager(request.id)
        else:
            return actor
    elif user_type == "customer":
        return Customer(request.id)
```

Ruby

For the purposes of this example, let’s assume that `AccountData` is a
resource that has an `account_id` attribute.

We can update our application code slightly to generate `AccountManager` users:

```

def user_from_id(id)
  case db.query('SELECT type FROM users WHERE id = ?', id)
  when 'internal'
    actor = InternalUser.new(id: id)
    if actor.role == 'account_manager'
      AccountManager.new(id: id)
    else
      actor
    end
  when 'customer'
    Customer.new(id: id)
  end
end
```

Java

Java example coming soon.

Node.js

For the purposes of this example, let’s assume that `AccountData` is a
resource that has an `accountId` attribute.

We can update our application code slightly to generate `AccountManager` users:

```

function userFromId(id) {
  const userType = db.query('SELECT type FROM users WHERE id = ?', id);
  if (userType === 'internal') {
    const actor = new InternalUser(id);
    if (actor.role() === 'account_manager') {
      return new AccountManager(id);
    } else {
      return actor;
    }
  } else if (userType === 'customer') {
    return new Customer(id);
  }
}

module.exports = { oso };
```

We’ve now successfully secured all three dashboards and customer account data.

## Summary

It is common to require different authorization logic for different types of application users. In this example,
we showed how to use different Actor types to represent different users in oso. We wrote policies with rules
that specialized on the type of Actor, and even added attributes to some actor types that we used in the policy.
We also demonstrated how inheritance can be used to match rules to multiple types of Actors.
