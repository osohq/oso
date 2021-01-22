---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/context
images: {}
path: /using-examples-context
title: Context
---

# Context

Allow rules take in an actor (which comes from authorization
logic) and a resource (which comes from mapping).  Sometimes
you need some additional context information about the environment to write
rules over.

## Context

For example, let’s say you have a policy like this:

```
allow(actor, _action, _resource) if role(actor, "admin");

```

Here we have a very simple allow rule that allows an actor to access any
resource if they are an admin.  Maybe we want to also let any actor access any
resource when the app is in development mode.  A typical way to flag that an app
is running in development or production mode would be to set an environment
variable, e.g. `ENV=development` or `ENV=production`.

How would we read that environment variable from our policy though?

We can use a application class that lets us directly access the environment
variables.

Python

```
import os
from oso import polar_class


@polar_class
class Env:
    @staticmethod
```

Ruby

```
require "oso"

OSO ||= Oso.new

class Env
  def self.var(variable)
    ENV[variable]
  end
end

OSO.register_class(Env)
```

Java

```
import java.util.Map;

public class Env {
  public static String var(String variable) {
    Map<String, String> env = System.getenv();
    return env.get(variable);
  }
}
```

Node.js

```
class Env {
  static var(variable) {
    return process.env[variable];
  }
}

oso.registerClass(Env);
```

The above class exposes a var method that reads the application’s environment
variables and returns the value asked for.  We can then register the class with
`register_class`, which will let us instantiate it in the policy.

We can add a new allow rule that allows an actor to access a resource if the
application is in development mode.

```

allow(_actor, _action, _resource) if Env.var("ENV") = "development";
```

## Summary

Application classes make it easy to expose any sort of application data to your
policy, including environment variables and request context. This simple pattern
lets you expose any kind of data you want to use in your policy, not just
`Actor` and `Resource` classes.
