"""Test advanced SQLAlchemy queries using features like joinedload, contains_eager,
and subquery.
"""
import pytest

import sqlalchemy
from sqlalchemy import Column, ForeignKey, Integer, select, String, create_engine
from sqlalchemy.orm import (
    declarative_base,
    joinedload,
    lazyload,
    selectinload,
    subqueryload,
    contains_eager,
    with_loader_criteria,
    Load,
    Session,
    relationship)

from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3
from sqlalchemy_oso.sqlalchemy_utils import (
    all_entities_in_statement,
    get_column_entities,
    get_joinedload_entities,
    to_class
)
from oso import Oso


pytestmark = pytest.mark.skipif(USING_SQLAlchemy_v1_3,
                                reason="Only runs on SQLAlchemy 1.4")


Base = declarative_base()


"""Test models

A => B => C
 \\=> A1
"""

class A(Base):
    __tablename__ = "a"

    id = Column(Integer, primary_key=True)
    data = Column(String)
    bs = relationship("B", backref="a")
    a1s = relationship("A1")


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


class A1(Base):
    __tablename__ = "a1"
    id = Column(Integer, primary_key=True)
    data = Column(String)
    a_id = Column(ForeignKey("a.id"))


@pytest.mark.parametrize('stmt,o', (
    (select(A), {A}),
    (select(A, B), {A, B}),
    (select(A.data, B.data), {A, B}),
    (select(A, B.data).join(B), {A, B}),
))
def test_get_column_entities(stmt, o):
    # Tested using new-style select API so that a session is not required.
    assert get_column_entities(stmt) == o


# TODO errors for wildcard, String.

@pytest.mark.parametrize('stmt,o', (
    (select(A), set()),
    (select(A).options(joinedload(A.bs)), {B}),
    (select(A).options(joinedload(A.bs).joinedload(B.cs)), {B, C}),
    (select(A).options(Load(A).joinedload("bs")), {B}),
    pytest.param(select(A).options(Load(A).joinedload("*")), set(),
                 marks=pytest.mark.xfail(reason="wildcard doesn't work")),
    pytest.param(select(A).options(joinedload("*")), set(),
                 marks=pytest.mark.xfail(reason="wildcard doesn't work")),
))
def test_get_joinedload_entities(stmt, o):
    assert set(map(to_class, get_joinedload_entities(stmt))) == o

@pytest.mark.parametrize('stmt,o', (
    pytest.param(select(A).options(joinedload("A.bs")), {B}, marks=pytest.mark.xfail(reason="String doesn't work")),
))
def test_get_joinedload_entities_str(stmt, o):
    assert set(map(to_class, get_joinedload_entities(stmt))) == o

# TODO test with lazy = "subquery", etc.
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


def test_subquery_joined():
    subquery = select(A).join(B).subquery(name='sub')
    subquery_aliased = sqlalchemy.orm.aliased(A, alias=subquery, flat=True, adapt_on_names=True)
    query_for_c = select(subquery_aliased).outerjoin(A1).options(
        contains_eager(A.a1s),
        contains_eager(A.bs, alias=subquery_aliased)
    )

    assert all_entities_in_statement(query_for_c) == {A, B, A1}


def test_with_loader_criteria_simple_alias():
    aliased = sqlalchemy.orm.aliased(A)
    query_for_a = select(aliased).options(
        with_loader_criteria(A, A.id == 1, include_aliases=True),
    )

    assert all_entities_in_statement(query_for_a) == {A}
    # Crude way of detecting filter on a.id in the generated query.
    assert 'a_1.id =' in str(query_for_a)


def test_with_loader_criteria_simple_subquery_no_alias():
    subquery = select(A).subquery(name='sub')
    query_for_a = select(subquery).options(
        with_loader_criteria(A, A.id == 1, include_aliases=True),
    )

    assert all_entities_in_statement(query_for_a) == {A}
    # Crude way of detecting filter on a.id in the generated query.
    assert 'a.id =' in str(query_for_a)


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


def test_lazy_load():
    from sqlalchemy_oso.session import AuthorizedSession
    oso = Oso()
    oso.register_class(A)
    oso.register_class(B)

    # Allow 1.
    oso.load_str('allow(_, _, a: A) if a.id = 0; allow(_, _, b: B) if b.id = 0;')

    # Ensure that running a lazy load properly applies authorization.
    engine = create_engine('sqlite://')
    Base.metadata.create_all(engine)

    with Session(bind=engine) as s, s.begin():
        a0 = A(id=0, data="0")
        b0 = B(id=0, a=a0)
        b1 = B(id=1, a=a0)
        s.add_all([a0, b0, b1])

    session = AuthorizedSession(bind=engine, oso=oso, user='u', checked_permissions={A: 'a', B: 'a'})
    with session.begin():
        a = session.query(A).one()
        bs = a.bs
        assert len(bs) == 1
