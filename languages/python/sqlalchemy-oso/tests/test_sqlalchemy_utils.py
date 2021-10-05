"""Test SQLAlchemy utilities.

The primary tests in this file confirm that `get_joinedload_entities` properly
loads all entities referenced in a query.
"""
import pytest

from sqlalchemy import Column, ForeignKey, Integer, select, String
from sqlalchemy.orm import (
    declarative_base,
    joinedload,
    lazyload,
    selectinload,
    subqueryload,
    Load,
    relationship)

from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3
from sqlalchemy_oso.sqlalchemy_utils import (
    all_entities_in_statement,
    get_column_entities,
    get_joinedload_entities)


pytestmark = pytest.mark.skipif(USING_SQLAlchemy_v1_3,
                                reason="Only runs on SQLAlchemy 1.4")


Base = declarative_base()


class A(Base):
    __tablename__ = "a"

    id = Column(Integer, primary_key=True)
    data = Column(String)
    bs = relationship("B")


class B(Base):
    __tablename__ = "b"
    id = Column(Integer, primary_key=True)
    a_id = Column(ForeignKey("a.id"))
    data = Column(String)
    cs = relationship("C")


class C(Base):
    __tablename__ = "c"
    id = Column(Integer, primary_key=True)
    b_id = Column(ForeignKey("b.id"))
    data = Column(String)


@pytest.mark.parametrize('stmt,o', (
    (select(A), {A}),
    (select(A, B), {A, B}),
    (select(A.data, B.data), {A, B}),
    (select(A, B.data).join(B), {A, B}),
))
def test_get_column_entities(stmt, o):
    # Tested using new-style select API so that a session is not required.
    assert get_column_entities(stmt) == o


@pytest.mark.parametrize('stmt,o', (
    (select(A), set()),
    (select(A).options(joinedload(A.bs)), {B}),
    (select(A).options(joinedload(A.bs).joinedload(B.cs)), {B, C}),
    (select(A).options(Load(A).joinedload("bs")), {B}),
    pytest.param(select(A).options(Load(A).joinedload("*")), set(),
                 marks=pytest.mark.xfail(reason="* doesn't work")),
))
def test_get_joinedload_entities(stmt, o):
    assert set(get_joinedload_entities(stmt)) == o


def test_default_loader_strategies():
    Base2 = declarative_base()
    class D(Base2):
        __tablename__ = "d"
        id = Column(Integer, primary_key=True)
        data = Column(String)
        es = relationship("E", lazy="joined")

    class E(Base2):
        __tablename__ = "e"
        id = Column(Integer, primary_key=True)
        data = Column(String)
        d_id = Column(ForeignKey('d.id'))
        fs = relationship("F", lazy="joined")

    class F(Base2):
        __tablename__ = "f"
        id = Column(Integer, primary_key=True)
        data = Column(String)
        e_id = Column(ForeignKey('e.id'))

    assert all_entities_in_statement(select(D, E)) == {D, E, F}
    assert all_entities_in_statement(select(E)) == {E, F}

# TODO test subquery, selectin. These are okay I believe because the
# compiles of the select in & subquery trigger separate with orm execute
# events, so we can't really test this way.
@pytest.mark.xfail(reason="idk how to test subquery yet")
@pytest.mark.parametrize('stmt,o', (
    (select(A).options(subqueryload(A.bs)), {A, B}),
))
def test_subquery(stmt, o):
    assert all_entities_in_statement(stmt) == o

@pytest.mark.xfail(reason="idk how to test selectin yet")
@pytest.mark.parametrize('stmt,o', (
    (select(A).options(selectinload(A.bs)), {A, B}),
))
def test_selectinload(stmt, o):
    assert all_entities_in_statement(stmt) == o

# TODO test m2m
# TODO test lazy load over joinedload
