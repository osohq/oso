from dataclasses import dataclass
import copy
from typing import Any, List
from abc import ABCMeta

from polar.test_helpers import *
from polar.polar import Polar


# This metaclass stuff is sad.

@dataclass
class Expense(metaclass=ABCMeta):
    name: str
    amount: int


class ExpressionField:
    def __init__(self, name, expression):
        self.names = [name]
        self.expression = expression

    def copy(self):
        e = ExpressionField(None, None)
        e.names = self.names
        e.expression = self.expression
        return e

    def push(self, name):
        self.names.append(name)

    def __getattr__(self, name):
        copy = self.copy()
        copy.push(name)
        return copy

    def __eq__(self, other):
        self.expression.equality(self, other)
        return True

@dataclass
class Equality:
    field_path: List[str]
    value: Any

    def __str__(self):
        return f"{field_path.join('.')} == {value}"

class PartialExpression:
    def __init__(self, cls):
        self._cls = cls
        self.expressions = []

    def __getattr__(self, name):
        return ExpressionField(name, self)

    def equality(self, field, other):
        self.expressions.append(Equality(field.names, other))

    def __eq__(self, other):
        self.expressions.append(Equality("self", other))

# HACK in a HACK
Expense.register(PartialExpression)

def test_partial_works():
    polar = Polar()

    polar.register_class(Expense)
    polar.load_str("""
        allow(actor, "view", expense: Expense) if
            actor.id = expense.user.id;
        """)

    expr = PartialExpression(Expense)
    result = next(polar.query_rule("allow", {'id': '1'}, "view", expr))
    assert expr.expressions[0].field_path == ['user', 'id']
    assert expr.expressions[0].value == '1'
