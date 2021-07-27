# Data Filtering

# We have some new stuff they have to define that is type and relationship information.
# We have some hooks they need to implement so we can fetch data.
# We have to have some top level call that initiates everything.

from typing import Any, List, Dict
from dataclasses import dataclass

from .expression import Expression
from .partial import Variable, Pattern

VALID_KINDS = ["parent"]

# Used so we know what fetchers to call and how to match up constraints.
@dataclass
class Relationship:
    kind: str
    other_type: str
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
    kind: str  # ["Eq", "In"]
    field: str
    value: Any  # Value or list of values.


@dataclass
class Constraints:
    cls: str
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

def var_name(var):
    return super(Variable, var).__str__()


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

            # I suppose I can use the type information to understand if the nested thing is a dict or
            # another embedded type or a relationship. For embedded dicts or other types I guess I'll
            # just have to treat them as path'd unifies.
            # foo.bar.baz = 12
            # Since there's no relationship there's no other way to do it really.

            # So what should walking a dot really do?
            # Also, what do we do if we get expressions that aren't about _this?

            assert var.operator == "Dot"
            assert len(var.args) == 2
            inner_var = var.args[0]
            inner_field = var.args[1]
            assert inner_var == Variable("_this")
            assert isinstance(inner_field, str)

            return (inner_field, field)



    def process_exp(self, exp):
        if exp.operator == "And":
            # @NOTE: This might be the place we actuially combine up a filterplan and return it
            # and maybe it'll get combined with other ones in outer ands, but something else
            # if there's an or.
            for arg in exp.args:
                self.process_exp(arg)
        elif exp.operator == "Dot":
            # Dot operators return a var that can be unified with. We just create new temp
            # vars for any dots.
            assert len(exp.args) == 2
            var = exp.args[0]
            field = exp.args[1]
            if isinstance(var, Expression):
                assert var.operator == "Dot"
                var = self.process_exp(var)
            assert isinstance(var, Variable)
            assert isinstance(field, str)
            # Create new variable and relate it to the current one.
            # Making sure two dots are the same is hard tho. How do we do that?
            # I guess we have to do more unifying if two vars are the same relationship to
            # another var?
            # There's also the potential name clash here which would be bad. Works for now.
            new_var = Variable(var_name(var) + "_dot_" + field)
            self.var_relationships.append((var_name(var), field, var_name(new_var)))
            return new_var

        elif exp.operator == "Isa":
            assert len(exp.args) == 2
            lhs = exp.args[0]
            rhs = exp.args[1]
            assert isinstance(lhs, Variable)
            name = var_name(lhs)
            assert isinstance(rhs, Pattern)
            # TODO: Handle fields in Patterns
            assert rhs.fields == {}
            tag = rhs.tag
            self.var_types.append((name, tag))
        elif exp.operator == "Unify":
            assert len(exp.args) == 2
            lhs = exp.args[0]
            rhs = exp.args[1]
            if isinstance(lhs, Expression):
                lhs = self.process_exp(lhs)
            if isinstance(rhs, Expression):
                rhs = self.process_exp(rhs)

            if isinstance(lhs, Variable) and isinstance(rhs, Variable):
                # Unifying two variables.
                self.var_cycles.append({var_name(lhs), var_name(rhs)})
            elif not (isinstance(lhs, Variable) or isinstance(rhs, Variable)):
                # What are you doing then? 1 = 1? who cares?
                assert False, "why?"
            else:
                # One of them is a variable, put it on the left.
                if isinstance(rhs, Variable):
                    lhs, rhs = rhs, lhs
                # Left side is a variable, right side is a value.
                self.var_values.append((var_name(lhs), rhs))



            #     # Only handle unification with values for now.
            #     # TODO: stuff like _this.something = this.bar.something_else
            #     # it's sort of like an additional join constraint to the defined relationship
            #     assert not isinstance(lhs, Expression)
            # elif isinstance(lhs, Expression):
            #     assert not isinstance(rhs, Expression)
            #     lhs, rhs = rhs, lhs
            # # We are setting a constraint that the variable rhs must be equal to the value lhs
            # value = lhs
            # relation, field = self.walk_dot(rhs)
            #
            # if relation is None:
            #     # Put the constraint on the fetcher.
            #     self.data_sets[self.sid].constraints.append(
            #         Constraint("Eq", field, value)
            #     )
            # else:
            #     # Put the constraint on a related fetcher
            #
            #     # relation must be a relationship on _this (for now)
            #     assert self.cls in self.polar.host.types
            #     typ = self.polar.host.types[self.cls]
            #     assert relation in typ
            #     rel = typ[relation]
            #     assert isinstance(rel, Relationship)
            #     assert rel.kind == "parent"
            #     assert rel.other_type in self.polar.host.fetchers
            #
            #     # Create or get a fetcher for the type. (This is maybe not a global and instead one for
            #     # every expression that then get combined (when they're in ANDs) or something like that.
            #     id = None
            #     if ("_this", relation) not in self.path_sets:
            #         id = self.next_id
            #         self.next_id += 1
            #         self.path_sets[("_this", relation)] = id
            #         self.data_sets[id] = Constraints(rel.other_type, [])
            #     else:
            #         id = self.path_sets[("_this", relation)]
            #
            #     # Put constrant on relation
            #     self.data_sets[id].constraints.append(Constraint("Eq", field, value))
            #     # Put in constraint on _this
            #     self.data_sets[self.sid].constraints.append(
            #         Constraint("In", rel.my_field, Attrib(rel.other_field, Result(id)))
            #     )
            #     self.dependencies.insert(0, id)

    def process_bindings(self, query_results):
        # Making a bunch of assumptions and restrictions for now.
        # Want something that works for simple queries that I can then expand.
        query_results = list(query_results)
        assert len(query_results) == 1, "Steve, next thing to do is handle OR but you are very close!"
        assert "bindings" in query_results[0]
        assert len(query_results[0]["bindings"]) == 1  # Only one variable in bindings.
        assert self.variable in query_results[0]["bindings"]
        exp = query_results[0]["bindings"][self.variable]
        assert isinstance(exp, Expression)
        assert exp.operator == "And"
        self.process_exp(exp)

    def collapse_vars(self):
        """
        Takes the results from processing bindings and collapses all the vars.
        Creates a var_id for every cycle and returns all the same info indexed by those ids.
        """
        # merge cycles
        joined_cycles = []
        for cycle in self.var_cycles:
            merged = False
            for joined_cycle in joined_cycles:
                if len(joined_cycle.intersection(cycle)) > 0:
                    joined_cycle = joined_cycle.union(cycle)
                    merged = True
                    break
            if not merged:
                joined_cycles.append(cycle)

        # Substitute in vars.
        variables = {}
        next_id = 0
        for cycle in joined_cycles:
            if "_this" in cycle:
                self.this_var = next_id
            variables[next_id] = cycle
            next_id += 1

        # Substitute lhs of relationship.
        for i, (v, f, cv) in enumerate(self.var_relationships):
            found = False
            for id, s in variables.items():
                if v in s:
                    self.var_relationships[i] = (id, f, cv)
                    found = True
                    break
            if not found:
                variables[next_id] = {v}
                self.var_relationships[i] = (next_id, f, cv)
                next_id += 1
        # Substitute rhs of relationship.
        for i, (v, f, cv) in enumerate(self.var_relationships):
            found = False
            for id, s in variables.items():
                if cv in s:
                    self.var_relationships[i] = (v, f, id)
                    found = True
                    break
            if not found:
                variables[next_id] = {cv}
                self.var_relationships[i] = (v, f, next_id)
                next_id += 1

        # If two relationships are the same field with the same lhs var then the
        # rhs vars are the same var.
        new_unifies = []
        for i, (v1, f1, cv1) in enumerate(self.var_relationships):
            for j, (v2, f2, cv2) in enumerate(self.var_relationships):
                if i != j and v1 == v2 and f1 == f2:
                    # This means cv1 and cv2 are the same variable.
                    new_unifies.append((cv1, cv2))
        # @TODO: There are absolutely bugs in here.
        # if we're turning 0 into 1 and then 0 into 2 it'll just blow up
        # not correctly turn 0 and 1 into 2. Have to write so many tests.
        # Just ignoring that for now, will address later.
        for (x, y) in new_unifies:
            variables[x] = variables[x].union(variables[y])
            del variables[y]

        # Re-substitute all the relationship vars now.
        for i, (v, f, cv) in enumerate(self.var_relationships):
            for (x, y) in new_unifies:
                if v == y:
                    self.var_relationships[i] = (x, f, cv)
                if cv == y:
                    self.var_relationships[i] = (v, f, x)

        # Ok, so now all the relationships should be using var ids that are as merged as they can be.
        var_relationships = self.var_relationships

        # I believe a var can only have one value, since we make sure there's a var for the dot lookup.
        # And if they had aliases they'd be collapsed
        # so it should be an error if foo.name = "steve" and foo.name = "gabe"

        # TODO: How are you gonna handle "in"
        var_values = {}
        for var, value in self.var_values:
            found = False
            for id, s in variables.items():
                if var in s:
                    if id in var_values:
                        assert var_values[id] == value
                    else:
                        var_values[id] = value
                    found = True
                    break
            if not found:
                variables[next_id] = {var}
                var_values[next_id] = value
                next_id += 1

        # A var can only have one type.
        var_types = {}
        for var, type in self.var_types:
            found = False
            for id, s in variables.items():
                if var in s:
                    if id in var_types:
                        assert var_types[id] == type
                    else:
                        var_types[id] = type
                    found = True
                    break
            if not found:
                variables[next_id] = {var}
                var_types[next_id] = type
                next_id += 1

        self.variables = variables
        self.var_relationships = var_relationships
        self.var_types = var_types
        self.var_values = var_values

    def this_id(self):
        this_id = None
        for id, s in self.variables.items():
            if "_this" in s:
                this_id = id
                break
        assert this_id is not None
        return this_id

    def constrain_var(self, var_id, var_type):
        if var_id in self.var_types:
            if var_type:
                assert var_type == self.var_types[var_id]
            else:
                var_type = self.var_types[var_id]
        type_info = None
        if var_type:
            for cls, ti in self.polar.host.types.items():
                if cls == var_type:
                    type_info = ti
                    break

        if var_id not in self.data_sets:
            self.data_sets[var_id] = Constraints(var_type, [])
            # @TODO: This probably is a bug, dependencies can be more complicated than just pushing
            # to the front when we find one.
            self.dependencies.insert(0, var_id)
        else:
            assert self.data_sets[var_id].cls == var_type

        for rel_var_id, field, rel_rel_id in self.var_relationships:
            if rel_var_id == var_id:
                if field in type_info:
                    rel = type_info[field]
                    if isinstance(rel, Relationship):
                        # Get constraints for the related var.
                        self.constrain_var(rel_rel_id, rel.other_type)
                        self.data_sets[var_id].constraints.append(Constraint("In", rel.my_field, Attrib(rel.other_field, Result(rel_rel_id))))
                        continue

                # Non relationship or unknown type info.
                # @TODO: Handle "in"
                assert rel_rel_id in self.var_values
                value = self.var_values[rel_rel_id]
                self.data_sets[var_id].constraints.append(Constraint("Eq", field, value))


    # Probably pass through the initial type too.
    def build_constraints(self, var_id):
        self.constrain_var(var_id, self.cls)

    def plan(self, query_results):
        self.next_id = 2
        self.data_sets = {}
        self.path_sets = {}
        self.sid = 1
        self.dependencies = []

        self.var_cycles = []
        self.var_relationships = []
        self.var_values = []
        self.var_types = []

        self.process_bindings(query_results)
        self.collapse_vars()

        this_id = self.this_id()

        self.build_constraints(this_id)
        var_id = this_id
        filter_order = self.sort_dependencies()
        return FilterPlan(self.data_sets, filter_order, var_id)


def process_constraints(polar, cls, variable, query_results):
    cls_name = polar.host.cls_names[cls]
    planner = FilterPlanner(polar, cls_name, variable)
    plan = planner.plan(query_results)
    return plan


def evaluate(polar, cls, variable, query_results):
    plan = process_constraints(polar, cls, variable, query_results)
    return filter_data(polar, plan)

# [
#     {
#         "bindings": {
#             "resource": Expression(
#                 And,
#                 [
#                     Expression(Isa, [Variable("_this"), Pattern(Repo, {})]),
#                     Expression(Unify, [Variable("_this"), Variable("_resource_143")]),
#                     Expression(Isa, [Variable("_resource_143"), Pattern(Repo, {})]),
#                     Expression(
#                         Unify,
#                         [
#                             Expression(Dot, [Variable("_resource_143"), "org"]),
#                             Variable("_parent_org_741"),
#                         ],
#                     ),
#                     Expression(Isa, [Variable("_parent_org_741"), Pattern(Org, {})]),
#                     Expression(
#                         Unify,
#                         [
#                             "osohq",
#                             Expression(Dot, [Variable("_parent_org_741"), "name"]),
#                         ],
#                     ),
#                 ],
#             )
#         },
#         "trace": None,
#     }
# ]

# 0 [_this, _resource_143], REPO,
# 1 [_resource_143.org, _parent_org_741], ORG, {name: osohq}

# You can just sort of treat dots as a var maybe, or something like that.

# Within an and, we wanna get all the vars, all the constraints on those vars and to know the relationships
# between those vars. Then I guess we sorta work up the chain and turn that into a filterplan.

# _this Repo
# = _this _resource_143
# _resource_143 Repo
# = _resource_143_dot_org _parent_org_741
# relation resource_143 . org is _resource_143
# _parent_org_741 Org
#
# = _parent_org_741_dot_name "osohq"
# relation _parent_org_741 . name is _parent_org_741_dot_name
#
# So you get a list of unifies on vars and a list of relationships.
# You can use the relationships to hopefully build the graph?

# So I'm just creating new vars and relationships as I traverse dot expressions.
# Then I need to figure out how to work back from that to figure out which are new fetches
# and which are constraints on fields.

# I guess first step is handle all the cycles with a new id.
# Then look at the relationships for the ids.
# Not sure how this combines with OR yet but gonna just get this query working first.
# For relationships
#   If they are just typed, I can add as constraints.
#   If they are real relationships, I can start to build this filter graph and sub in that
# it's like a result or something.
# I think this can work.
