from dataclasses import dataclass
from oso import Oso

from oso import Variable

# oso = Oso()


# @dataclass
# class Bar:
#     id: str


# oso.register_class(Bar)

# policy_a = """
# resource(_type: Bar, "bar", actions, roles) if
#     actions = ["wave"] and
#     roles = {
#         member: {
#             permissions: ["wave"]
#         }
#     };

# actor_has_role_for_resource("steve", "member", resource: Bar) if
#     resource.id = "hello";

# allow(actor, action, resource) if
#     role_allows(actor, action, resource);
# """
# oso.load_str(policy_a)
# oso.enable_roles()

# resource = Bar(id="hello")
# assert oso.is_allowed("steve", "wave", resource)


# def print_em(results):
#     if not results:
#         print("no results")
#     for result in results:
#         bindings = result["bindings"]
#         resource = bindings["resource"]
#         for _, v in bindings.items():
#             print(v)


# results = list(oso.query('allow("steve", "wave", resource)', accept_expression=True))
# print_em(results)
# results = list(oso.query('allow("steve", "cry", resource)', accept_expression=True))
# print_em(results)

# # So, we do get back some expressions about _this.id == "hello". I dunno if this is verry realistic of a policy
# # but maybe a place to start.

# # Flow would I guess be something like this? (though most of it would ideally be in the core)

# # Query with resource unbound, concrete actor and action.
# # Take expression and match it to known query function prototypes (or something).
# # take results and run the policy on them.

# # start over lol

# oso = Oso()


# @dataclass
# class Bar:
#     id: str


# oso.register_class(Bar)

# hello_resource = Bar(id="hello")


# class API:
#     @staticmethod
#     def get_roles(actor):
#         if actor == "steve":
#             return [{"name": "member", "resource": hello_resource}]


# oso.register_constant(API, "API")


# policy_a = """
# resource(_type: Bar, "bar", actions, roles) if
#     actions = ["wave"] and
#     roles = {
#         member: {
#             permissions: ["wave"]
#         }
#     };

# actor_has_role_for_resource(actor, role_name, resource) if
#     role in API.get_roles(actor) and
#     role.name = role_name and role.resource = resource;

# allow(actor, action, resource) if
#     role_allows(actor, action, resource);
# """
# oso.load_str(policy_a)
# oso.enable_roles()

# assert oso.is_allowed("steve", "wave", hello_resource)


# def print_em(results):
#     if not results:
#         print("no results")
#     for result in results:
#         bindings = result["bindings"]
#         resource = bindings["resource"]
#         for _, v in bindings.items():
#             print(v)


# results = list(oso.query('allow("steve", "wave", resource)', accept_expression=True))
# print_em(results)
# results = list(oso.query('allow("steve", "cry", resource)', accept_expression=True))
# print_em(results)

# This maybe could work.

# It sort of breaks down when there are parent relationships. There's no real way
# to map those to some easy to understand constraints without constraining
# the way the rule is written, for example
#
# parent_child(parent_org, repo: Repo) if
#       org.get_parent_repo() = repo;
# This totally cuts us out of the whole process of trying to understand what's
# happening. It's an external method on an unbound and just isn't really reversabe.

# There's an interesting case which is sort of what the sqlalchemy roles library
# does is that there's a way to get a parent, but also a way to get all
# children. That's what you need to do the data filtering version where you
# go from a role you have the things you have access to.

# This is actually kind of a neat idea. Instead of basing data filtering on
# polar, we could just base it on the roles model since there's a straitforward
# understanding of what the roles model looks like in the data filtering case.
# They would then have to define just things like get_children() and get_foo_by_id()

# That fully restricts what polar can do. We'd be forcing users to just stick to using
# the roles library and nothing else. That sucks too.

# It's probably better to just come up with an api for constraints. One that looks like a series of filters
# that are applied.

# We can't realy on anything about the parent relationships being consistent. They could be an accessor that returns a whole thing.
# foo.bar
# They could be some lookup by id
# API.get_bar(foo.bar_id)
# They could be something else entirely
# bar in ALL_BARS and bar.id = foo.bar_id
# In all of these cases we're sorta wanting a join. In the first case maybe it's an embedded object or maybe it's actually
# an orm magic property that fetches a related object. There's like no way for polar to know any of this right now.
# Is there some format we can put the constraints into where the user can just handle what they need to handle?

# The tree of filters idea was probably the best one. There's still this implicit notion that we're filtering a collection of the
# resource passed in. We also know that any other objects are going to come from properties on the objects or from some sort
# of external method call. That means that we can maybe still order this thing in a way where joins make sense. It's just gonna mean
# that external methods come accross as a kind of special constraint that then there has to be an easy way for the user code to handle
# it.

# lets take 3 forms of roles queries and see what we can come up with.
# Note, getting ALL roles usually only depends on the actor so it's not really a problem for data filtering.
# These relationships however could be because they'll be methods on an unbound thing. That's why they'll have to
# come through as constraints.

oso = Oso()


@dataclass
class Bar:
    id: str


@dataclass
class Foo:
    id: str
    bar: Bar


oso.register_class(Bar)
oso.register_class(Foo)

hello_bar = Bar(id="hello")
something_foo = Foo(id="something", bar=hello_bar)


class API:
    @staticmethod
    def get_roles(actor):
        if actor == "steve":
            return [
                {"name": "member", "resource": hello_bar},
            ]


oso.register_constant(API, "API")

policy_a = """
resource(_type: Bar, "bar", actions, roles) if
    actions = ["wave"] and
    roles = {
        member: {
            permissions: ["wave"],
            implies: ["foo:member_foo"],
        }
    };

resource(_type: Foo, "foo", actions, roles) if
    actions = ["wave_foo"] and
    roles = {
        member_foo: {
            permissions: ["wave_foo"]
        }
    };

actor_has_role_for_resource(actor, role_name, resource) if
    role in API.get_roles(actor) and
    role.name = role_name and role.resource = resource;

# Directly access the parent as a property
parent_child(parent_bar, foo: Foo) if
    foo.bar = parent_bar and
    parent_bar matches Bar;

allow(actor, action, resource) if
    role_allows(actor, action, resource);
"""
oso.load_str(policy_a)
oso.enable_roles()

assert oso.is_allowed("steve", "wave", hello_bar)
assert oso.is_allowed("steve", "wave_foo", something_foo)


def print_em(results):
    if not results:
        print("no results")
    for result in results:
        bindings = result["bindings"]
        resource = bindings["resource"]
        for _, v in bindings.items():
            print(v)


results = list(oso.query('allow("steve", "wave", resource)', accept_expression=True))
print_em(results)

# Roles knows what role type can apply based on the action and then matches up with any
# of the assigned roles that the user has to come up with this constraint which basically says
# that the resource is a Bar and has the id "hello"
#
# Expression(And, [
#   Expression(Isa, [Variable('_this'), Pattern(Bar, {})]),
#   Expression(Unify, [Variable('_this'), Bar(id='hello')])
# ])

results = list(
    oso.query(
        'allow("steve", "wave_foo", resource)',
        accept_expression=True,
    )
)
print_em(results)

# Hmm, this doesn't even come through at all with constraints.
# wtf should it look like?
# Well, role allows will fetch all the roles first which will work because they only depend on the
# actor. It knows the role the user can have is either member_foo on foo OR member on bar
# then it should match those to the roles the user has assigned
# and then should hopefully spit out somthing that looks like
# resource is a Foo and resource.bar is a bar and resource.bar has id "hello"
