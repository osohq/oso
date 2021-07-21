# Data Filtering

# We have some new stuff they have to define that is type and relationship information.
# We have some hooks they need to implement so we can fetch data.
# We have to have some top level call that initiates everything.

from typing import Any, List, Dict
from dataclasses import dataclass

from .expression import Expression
from .partial import Variable

VALID_KINDS = ["parent"]

# Used so we know what fetchers to call and how to match up constraints.
@dataclass
class Relationship:
    kind: str
    other_type: Any
    my_field: str
    other_field: str


# This is a first pass at what kind of thing I want to operate on.
# I'm calling it a "FilterPlan" but what it is is basically computes
# all the constraints into the various collections data has to be fetched from
# and with what constraints you have to fetch it. Those constraints could come
# from other collections. There's a dependency graph of which collections depend on
# other ones and we can topologically sort that so we know what order to fetch data in.
# This sorted order is provided.

# That way evaluation is easy, just fetch each collection, substituting in the results as you go
# when other collections need them.

# This is probably too simplistic and will have to evolve as I try this out on more and more
# valid polar but so far is a promising start. (eg there are probably many of these and ways to do
# unions or differences or all kinds of other stuff)
# We should be able to emit something like this right out of the core and then only have to implement
# the fetching and collecting logic in the language library.

# Id for a data set that will be fetched.
@dataclass
class Result:
    id: int


# Lets us express that we're looking up an attribute on a result.
@dataclass
class Attrib:
    key: str
    of: Result


@dataclass
class Constraint:
    kind: str  # ["eq", "in"]
    field: str
    value: Any  # Value or list of values.


@dataclass
class Constraints:
    cls: Any
    constraints: List[Constraint]


@dataclass
class FilterPlan:
    """
    Hopefully the thing we can get the core to spit out. It's like the plan
    for how to do the data filtering.
    """

    data_sets: Dict[int, Constraints]
    resolve_order: List[int]
    result_set: int


# For now my fetcher functions accept a Constraints object but each Constraint value has to be grounded
# which means they have to be actual data, not a reference to another data set.
# This is fine because I know the dependency order so the way to evaluate this is you start with the first
# collection, save it's results, and then for each other collection you can substitute in the results
# before you call the fetching function.
def ground_constraints(polar, results, filter_plan, constraints):
    # Walk the constraints substituting in any results
    # Since we walk in a dependency order we should have any results mentioned already
    for constraint in constraints.constraints:
        attrib = None
        if isinstance(constraint.value, Attrib):
            attrib = constraint.value.key
            constraint.value = constraint.value.of
        if isinstance(constraint.value, Result):
            constraint.value = results[constraint.value.id]
        if attrib is not None:
            constraint.value = [getattr(v, attrib) for v in constraint.value]


def filter_data(polar, filter_plan):
    results = {}
    for id in filter_plan.resolve_order:
        constraints = filter_plan.data_sets[id]
        # Substitute in fetched results.
        ground_constraints(polar, results, filter_plan, constraints)
        # Fetch the data.
        fetcher = polar.host.fetchers[constraints.cls]
        results[id] = fetcher(constraints)
    return results[filter_plan.result_set]


# The hardest part of this is taking the expressions in the bindings that come out of the core
# and turning them into my new format. Eventually (once this is all figured out) this part should
# be easy to port to rust and put into the core.

# We want to take the results from the partial query and return our FilterPlan thing.
# [
#     {
#         "bindings": {
#             "resource": Expression(
#                 And,
#                 [
#                     Expression(Isa, [Variable("_this"), Pattern(Foo, {})]),
#                     Expression(
#                         Unify, [True, Expression(Dot, [Variable("_this"), "is_fooey"])]
#                     ),
#                 ],
#             )
#         },
#         "trace": None,
#     }
# ]
# =>
# FilterPlan({1: Constraints(Foo, [Constraint("Eq", "is_fooey", True)])}, [1], 1)

# [
#     {
#         "bindings": {
#             "resource": Expression(
#                 And,
#                 [
#                     Expression(Isa, [Variable("_this"), Pattern(Foo, {})]),
#                     Expression(
#                         Unify,
#                         [
#                             True,
#                             Expression(
#                                 Dot,
#                                 [
#                                     Expression(Dot, [Variable("_this"), "bar"]),
#                                     "is_cool",
#                                 ],
#                             ),
#                         ],
#                     ),
#                     Expression(
#                         Unify, [True, Expression(Dot, [Variable("_this"), "is_fooey"])]
#                     ),
#                 ],
#             )
#         },
#         "trace": None,
#     }
# ]
# =>
# FilterPlan(
#     {
#         1: Constraints(
#             Foo,
#             [
#                 Constraint("In", "bar_id", Attrib("id", Result(2))),
#                 Constraint("Eq", "is_fooey", True),
#             ],
#         ),
#         2: Constraints(Bar, [Constraint("Eq", "is_cool", True)]),
#     },
#     [2, 1],
#     1,
# )


class FilterPlanner:
    def __init__(self, polar, cls, variable):
        self.polar = polar
        self.cls = cls
        self.variable = variable

    def sort_dependencies(self):
        # topologically sort all the dependencies so we have an
        # order we can execute the fetches in
        return self.dependencies

    # dot _this foo
    # dot (dot _this bar) foo
    def walk_dot(self, exp):
        # Return what the unify needs to know to put this constraint
        # on the right fetch.
        assert exp.operator == "Dot"
        assert len(exp.args) == 2
        var = exp.args[0]
        field = exp.args[1]
        if isinstance(var, Variable):
            # TODO: Keeping track of other variables.
            assert var == Variable("_this")
            assert isinstance(field, str)
            return (None, field)
        if isinstance(var, Expression):
            # For now, this is a hack
            # I only support 2 levels of nested fields and the first level HAS
            # to be a relationship.
            # Obviously that's not always the case and we need to just walk the path
            # and figure out where the constraint goes.
            assert var.operator == "Dot"
            assert len(var.args) == 2
            inner_var = var.args[0]
            inner_field = var.args[1]
            assert inner_var == Variable("_this")
            assert isinstance(inner_field, str)

            return (inner_field, field)

    def process_exp(self, exp):
        if exp.operator == "And":
            for arg in exp.args:
                self.process_exp(arg)
        elif exp.operator == "Isa":
            # Ignoring for now, probably shouldn't
            pass
        elif exp.operator == "Unify":
            assert len(exp.args) == 2
            lhs = exp.args[0]
            rhs = exp.args[1]
            if isinstance(rhs, Expression):
                # Only handle unification with values for now.
                # TODO: stuff like _this.something = this.bar.something_else
                # it's sort of like an additional join constraint to the defined relationship
                assert not isinstance(lhs, Expression)
            elif isinstance(lhs, Expression):
                assert not isinstance(rhs, Expression)
                lhs, rhs = rhs, lhs
            # We are setting a constraint that the variable rhs must be equal to the value lhs
            value = lhs
            relation, field = self.walk_dot(rhs)

            if relation is None:
                # Put the constraint on the fetcher.
                self.data_sets[self.sid].constraints.append(Constraint("Eq", field, value))
            else:
                # Put the constraint on a related fetcher

                # relation must be a relationship on _this (for now)
                assert self.cls in self.polar.host.types
                typ = self.polar.host.types[self.cls]
                assert relation in typ
                rel = typ[relation]
                assert isinstance(rel, Relationship)
                assert rel.kind == "parent"
                assert rel.other_type in self.polar.host.fetchers

                # Create or get a fetcher for the type. (This is maybe not a global and instead one for
                # every expression that then get combined (when they're in ANDs) or something like that.
                id = None
                if ("_this", relation) not in self.path_sets:
                    id = self.next_id
                    self.next_id += 1
                    self.path_sets[("_this", relation)] = id
                    self.data_sets[id] = Constraints(rel.other_type, [])
                else:
                    id = self.path_sets[("_this", relation)]

                # Put constrant on relation
                self.data_sets[id].constraints.append(Constraint("Eq", field, value))
                # Put in constraint on _this
                self.data_sets[self.sid].constraints.append(Constraint("In", rel.my_field, Attrib(rel.other_field, Result(id))))
                self.dependencies.insert(0, id)


    def process_bindings(self, query_results):
        # Making a bunch of assumptions and restrictions for now.
        # Want something that works for simple queries that I can then expand.
        query_results = list(query_results)
        assert len(query_results) == 1
        assert "bindings" in query_results[0]
        assert len(query_results[0]["bindings"]) == 1  # Only one variable in bindings.
        assert self.variable in query_results[0]["bindings"]
        exp = query_results[0]["bindings"][self.variable]
        assert isinstance(exp, Expression)
        assert exp.operator == "And"
        self.process_exp(exp)

    def plan(self, query_results):
        self.next_id = 2
        self.data_sets = {1: Constraints(self.cls, [])}
        self.path_sets = {("_this",): 1}
        self.sid = 1
        self.dependencies = [1]

        self.process_bindings(query_results)

        filter_order = self.sort_dependencies()

        return FilterPlan(self.data_sets, filter_order, 1)


def process_constraints(polar, cls, variable, query_results):
    planner = FilterPlanner(polar, cls, variable)
    plan = planner.plan(query_results)
    return plan


def evaluate(polar, cls, variable, query_results):
    plan = process_constraints(polar, cls, variable, query_results)
    return filter_data(polar, plan)
