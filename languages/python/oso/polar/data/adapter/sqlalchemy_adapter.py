from functools import reduce

from sqlalchemy.inspection import inspect
from sqlalchemy.orm import Session
from sqlalchemy.sql import false, true

from ..filter import Projection
from .adapter import DataAdapter


class SqlAlchemyAdapter(DataAdapter):
    def __init__(self, session: Session) -> None:
        self.session = session

    def build_query(self, filter):
        types = filter.types

        def re(q, rel):
            typ = types[rel.left]
            rec = typ.fields[rel.name]
            left = typ.cls
            right = types[rec.other_type].cls
            return q.join(
                right, getattr(left, rec.my_field) == getattr(right, rec.other_field)
            )

        query = reduce(re, filter.relations, self.session.query(filter.model))
        disj = reduce(
            lambda a, b: a | b,
            [
                reduce(
                    lambda a, b: a & b,
                    [SqlAlchemyAdapter.sqlize(conj) for conj in conjs],
                    true(),
                )
                for conjs in filter.conditions
            ],
            false(),
        )
        return query.filter(disj).distinct()

    def execute_query(self, query):
        return query.all()

    @staticmethod
    def sqlize(cond):
        op = cond.cmp
        lhs = SqlAlchemyAdapter.add_side(cond.left)
        rhs = SqlAlchemyAdapter.add_side(cond.right)
        if op == "Eq":
            return lhs == rhs
        elif op == "Neq":
            return lhs != rhs
        elif op == "In":
            return lhs in rhs
        elif op == "Nin":
            return lhs not in rhs

    @staticmethod
    def add_side(side):
        if isinstance(side, Projection):
            source = side.source
            field = side.field or inspect(source).primary_key[0].name
            return getattr(source, field)
        elif inspect(type(side), raiseerr=False) is not None:
            return getattr(side, inspect(type(side)).primary_key[0].name)
        else:
            return side
