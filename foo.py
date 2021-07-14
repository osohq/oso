from typing import Any, ClassVar
from dataclasses import dataclass
from oso import Oso

from oso import Variable

oso = Oso()

# Still very tbd what this is but would support
# the normal things orms support.
# NOTE: Syntax, how this is defined, where it comes from (scraped from orm / polar / both) can all come later.
@dataclass
class Relationship:
    kind: str
    other_type: Any


@dataclass
class Bar:
    id: str
    is_cool: bool


@dataclass
class Foo:
    id: str
    bar_id: str

    # Pretty sure this doesn't get defined like this but defined some other way.
    bar: ClassVar[Relationship] = Relationship(kind="one-to-many", other_type=Bar)


oso.register_class(Bar)
oso.register_class(Foo)

hello_bar = Bar(id="hello", is_cool=True)
goodbye_bar = Bar(id="goodbye", is_cool=False)
something_foo = Foo(id="something", bar_id="hello")

bars = [hello_bar, goodbye_bar]
foos = [something_foo]

# foo parents are things in bar joined on this...
# refer to parent on foo with property bar

policy_a = """
allow("steve", "get", resource: Foo) if
    resource.is_dope = true and
    bar = resource.bar and
    bar.is_cool = true;
"""
oso.load_str(policy_a)
oso.repl()

# This policy won't work anymore because bar isn't actuially a field on Foo.
# orms implement some magic for this and now we will too.

# We'll have the user define a hook for their relationships (actually two hooks, see below)
# The first hook will be to walk the relationship in the normal way for normal queries.
def foo_get_bar(foo):
    # if it's an orm object, maybe this is just
    # return foo.bar
    for bar in bars:
        if bar.id == foo.bar_id:
            return bar
    return None


# When polar hits a property that it knows is a relationship, it just calls the external method
# that's defined. For orm libraries we can magically hook all this up to just use the orm but
# for other situations like this the user can write the function.

# This looks basically the same but where it really shines is the data filtering case which can be
# made a lot better now that polar knows about types and relationships.

# The constraints that come out of a data filtering version of this policy look like this.
# resource = resource = Expression(
#     And,
#     [
#         Expression(Isa, [Variable("_this"), Pattern(Foo, {})]),
#         Expression(
#             Unify,
#             [
#                 True,
#                 Expression(
#                     Dot, [Expression(Dot, [Variable("_this"), "bar"]), "is_cool"]
#                 ),
#             ],
#         ),
#     ],
# )

# (and
#   (isa _this Foo)
#   (= true (. (. _this bar) is_cool)
# )

# What we are doing is data filtering, so we want all the Foos that match these
# constraints.
# The foos that match are the ones where _this.bar matches something.
# the bars that match are the ones where _bar.is_cool = true.
# We can then invert the query, starting with the bars and working back to the foos.

# Get all bars where is_cool = true
# Use the inverse of the relationship to get all the foos.

# You can imagine there's some way to invert this query, then in evaluating it you're hitting two hooks.

# Seems like we'll need a top level fetcher for each type.
def get_bars(is_cool):
    for bar in bars:
        if bar.is_cool:
            yield bar


# And we'll need an inverse for the relationships.
# Instead of taking a single foo and returning a single parent bar.
# We take many bars and for each one return all their foos.
def get_foos_for_bars(bars):
    # This isn't great beacause I have no index in this example
    # in real life would be better.
    # also obviously api of these functions is tbd.
    for bar in bars:
        for foo in foos:
            if foo.bar_id == bar.id:
                yield (bar, foo)


# Now we can evaluate the whole data filter and call these hooks to get all the foos the
# user is allowed to see.

# This would be like the "low level" way to use it since it's just writing hooks for types and
# relationships and all the actual joining would happen in polar.
# For use cases like "direct to sql" we should be able to emit sql directly based on the
# relationship information which means this should support both use cases really well.


# Plan

# Do a proof of concept
# . Hack out a working version of polar using relationships to call hooks and the flow of it.
# . Just define relationships and callbacks however you have to to get it working.
# Take what we learn and do some designing
# . Once we've validated that the idea can work, we are in a much better place to think about apis.
# . What the user api looks like for defining (types / relationships / data fetching) hooks themselves.
# . What the "scraped from the orm" api looks like for implementing an orm library.
# . What different data is needed for direct to sql, how that relates to the first 2 apis. Make the whole thing
#     a unified story that makes sense.

# I think a wow hack prototype is the best way to start though because it's going to expose problems and other
# things we haven't thought about yet but I think this is a really promising idea to actually try.
