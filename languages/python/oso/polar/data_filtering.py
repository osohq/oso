# Data Filtering

# We have some new stuff they have to define that is type and relationship information.
# We have some hooks they need to implement so we can fetch data.
# We have to have some top level call that initiates everything.

# Should I start with the user side, in the form of a test? Yes, that seems to be a great idea.
# And... Should that test use roles perhaps?????? Sure could.


# So we have some relationship types or whatever that's needed to get us some info
# We have the callback methods registered on something.
# We have the actuial evaluation of everything.

# Should the evaluation be like a different VM in the core? It sort of does the same
# thing maybe as the normal vm where it talks in events?

# Other ways to do it, maybe all the evaluation does happen in the host language but we
# just have to implement it for each language. It's not all that much maybe?

# Maybe it is the visitor pattern thing that we had before and you can fill out
# different parts of the expression?

# If the fetching is events, maybe then it all runs in the core and calls out?
# How would you do other apis if you did it like that though?

# Just going all python to start anyway so don't worry about it dawg just hack.

from typing import Any, List, Dict
from dataclasses import dataclass

from .expression import Expression

VALID_KINDS = ["parent"]


@dataclass
class Relationship:
    kind: str
    other_type: Any
    my_field: str
    other_field: str


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
#                 ],
#             )
#         },
#         "trace": None,
#     }
# ]

# There's a good chance this whole thing can go in the core and just talk with events.
# First try to get something working I'm going to do a direct recursive interpreter.
# Second try I'll "compile" the expressions down into just the list of things to do to filter the data.


# Here's the thing, this is like another whole polar evaluator. What a waste of time to write a full interpreter
# for polar twice. Can all of this stuff just be figured out in the vm while evaluating the first time? I hope so.

# Really trying to just hack my way into understanding this problem.

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


# Only pass the constraints to the fetcher that they can handle.
# So only field constraints (= and in) and values.

# There's the preprocess step, which can hopefully go into the core.
# Then there's this evaluate piece that actually calls fetchers to get all the data.


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


# Result of interpreting is gonna be like some query plan data
# It's gonna be an order to run things in, and other things like transforms to apply?
# I think.

# bars: Constraint(Bar, [is_cool = true]
# foos: Constraint(Foo, [bar_id in [Result_map(bars, "id")]]


class Interpreter:
    def __init__(self, polar, cls, variable):
        self.polar = polar
        self.filtering_class = cls
        self.filtering_variable = variable

    def eval_exp(self, exp):
        pass

    def eval_unify(self, args):
        assert len(args) == 2
        # Could have values or expressions on either side.
        if isinstance(args[0], Expression):
            if isinstance(args[1], Expression):
                raise Exception("TODO")
            first = self.eval_exp(args[0])
            second = args[1]
        elif isinstance(args[1], Expression):
            first = self.eval_exp(args[1])
            second = args[0]
        pass

    def eval_and(self, args):
        # Combine all fetch constraints for the and.
        constraints = {}
        for exp in args:
            op = exp.operator
            if op == "Isa":
                pass
            elif op == "Unify":
                self.eval_unify(exp.args)
            else:
                raise Exception("TODO")
        return {}

    def eval_result(self, query_result):
        assert "bindings" in query_result
        assert len(query_result["bindings"]) == 1
        assert self.filtering_variable in query_result["bindings"]
        exp = query_result["bindings"][self.filtering_variable]
        assert isinstance(exp, Expression)
        assert exp.operator == "And"

        # Get all the constraints.
        constraints = self.eval_and(exp.args)

        # Fetch the data
        # return it
        return []

    def eval(self, query_results):
        query_results = list(query_results)
        assert len(query_results) == 1
        result_objs = []
        for query_result in query_results:
            objs = self.eval_result(query_result)
            result_objs.extend(objs)
        # TODO: No duplicates.
        return result_objs


# Here is the most dumb simple interepreter I can write.
# If this works, everything is gonna be just fine.
def evaluate(polar, cls, variable, query_results):
    interp = Interpreter(polar, cls, variable)
    return interp.eval(query_results)
