# type: ignore
"""Test advanced SQLAlchemy queries using features like joinedload, contains_eager,
and subquery.

See: https://docs.sqlalchemy.org/en/14/orm/loading_relationships.html
"""
# Unfortunately we ignore types in this module because `sqlalchemy-stubs` is not
# yet updated for 1.4.

# N.B: This test only runs on SQLAlchemy 1.4. Loading it on 1.3 is disabled in
# conftest.py because even running the imports requires SQLAlchemy 1.4.

import pytest
import sqlalchemy
from oso import Oso
from sqlalchemy import Column, ForeignKey, Integer, String, create_engine, select
from sqlalchemy.orm import (
    Load,
    Session,
    contains_eager,
    declarative_base,
    joinedload,
    relationship,
    selectinload,
    subqueryload,
    with_loader_criteria,
)

from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3
from sqlalchemy_oso.session import AuthorizedSession
from sqlalchemy_oso.sqlalchemy_utils import (
    all_entities_in_statement,
    get_column_entities,
    get_joinedload_entities,
    to_class,
)

pytestmark = pytest.mark.skipif(
    USING_SQLAlchemy_v1_3, reason="Only runs on SQLAlchemy 1.4"
)


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


@pytest.fixture
def engine():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)
    return engine


@pytest.fixture
def test_data(engine):
    with Session(bind=engine) as s, s.begin():
        a0 = A(id=0, data="0")
        b0 = B(id=0, a=a0)
        b1 = B(id=1, a=a0)
        s.add_all([a0, b0, b1])


@pytest.mark.parametrize(
    "stmt,o",
    (
        (select(A), {A}),
        (select(A, B), {A, B}),
        (select(A.data, B.data), {A, B}),
        (select(A, B.data).join(B), {A, B}),
    ),
)
def test_get_column_entities(stmt, o):
    # Tested using new-style select API so that a session is not required.
    assert get_column_entities(stmt) == o


# TODO better errors for wildcard, String.
@pytest.mark.parametrize(
    "stmt,o",
    (
        (select(A), set()),
        (select(A).options(joinedload(A.bs)), {B}),
        (select(A).options(joinedload(A.bs).joinedload(B.cs)), {B, C}),
        (select(A).options(Load(A).joinedload("bs")), {B}),
        pytest.param(
            select(A).options(Load(A).joinedload("*")),
            set(),
            marks=pytest.mark.xfail(reason="wildcard doesn't work"),
        ),
        pytest.param(
            select(A).options(joinedload("*")),
            set(),
            marks=pytest.mark.xfail(reason="wildcard doesn't work"),
        ),
    ),
)
def test_get_joinedload_entities(stmt, o):
    assert set(map(to_class, get_joinedload_entities(stmt))) == o


@pytest.mark.parametrize(
    "stmt,o",
    (
        pytest.param(
            select(A).options(joinedload("A.bs")),
            {B},
            marks=pytest.mark.xfail(reason="String doesn't work"),
        ),
    ),
)
def test_get_joinedload_entities_str(stmt, o):
    assert set(map(to_class, get_joinedload_entities(stmt))) == o


def test_default_loader_strategies_all_entities_in_statement():
    """Test that all_entitites_in_statement finds default "joined" entities."""
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
        d_id = Column(ForeignKey("d.id"))
        fs = relationship("F", lazy="joined")

    class F(Base2):
        __tablename__ = "f"
        id = Column(Integer, primary_key=True)
        data = Column(String)
        e_id = Column(ForeignKey("e.id"))

    assert all_entities_in_statement(select(D, E)) == {D, E, F}
    assert all_entities_in_statement(select(E)) == {E, F}


@pytest.mark.parametrize("strategy", ("joined", "subquery", "selectin", "select"))
def test_default_loader_strategies(engine, strategy):
    """Test that default loader strategies are authorized correctly by running a query."""
    Base2 = declarative_base()

    class A1(Base2):
        __tablename__ = "a1"
        id = Column(Integer, primary_key=True)
        bs = relationship("B1", lazy=strategy, backref="a")

    class B1(Base2):
        __tablename__ = "b1"
        id = Column(Integer, primary_key=True)
        a_id = Column(ForeignKey("a1.id"))

    Base2.metadata.create_all(bind=engine)

    with Session(bind=engine) as s, s.begin():
        a0 = A1(id=0)
        a1 = A1(id=1)
        b0 = B1(id=0, a=a0)
        b1 = B1(id=1, a=a0)
        s.add_all([a0, a1, b0, b1])

    oso = Oso()
    oso.register_class(A1)
    oso.register_class(B1)
    oso.load_str("allow(_, _, a: A1) if a.id = 0; allow(_, _, b: B1) if b.id = 0;")

    with AuthorizedSession(
        bind=engine, oso=oso, checked_permissions={A1: "read", B1: "read"}, user="u"
    ) as auth_session, auth_session.begin():
        a = auth_session.query(A1).one()
        assert a.id == 0
        assert len(a.bs) == 1
        assert a.bs[0].id == 0


@pytest.mark.parametrize("strategy", ("joined", "subquery", "selectin", "select"))
def test_default_loader_strategies_no_auth(engine, strategy):
    """Sanity check of above."""
    Base2 = declarative_base()

    class A1(Base2):
        __tablename__ = "a1"
        id = Column(Integer, primary_key=True)
        bs = relationship("B1", lazy=strategy, backref="a")

    class B1(Base2):
        __tablename__ = "b1"
        id = Column(Integer, primary_key=True)
        a_id = Column(ForeignKey("a1.id"))

    Base2.metadata.create_all(bind=engine)

    with Session(bind=engine) as s, s.begin():
        a0 = A1(id=0)
        a1 = A1(id=1)
        b0 = B1(id=0, a=a0)
        b1 = B1(id=1, a=a0)
        s.add_all([a0, a1, b0, b1])

    with Session(bind=engine) as session, session.begin():
        a = session.query(A1).first()
        assert a.id == 0
        assert len(a.bs) == 2
        assert a.bs[0].id == 0


def test_subquery_joined():
    subquery = select(A).join(B).subquery(name="sub")
    subquery_aliased = sqlalchemy.orm.aliased(
        A, alias=subquery, flat=True, adapt_on_names=True
    )
    query_for_c = (
        select(subquery_aliased)
        .outerjoin(A1)
        .options(contains_eager(A.a1s), contains_eager(A.bs, alias=subquery_aliased))
    )

    assert all_entities_in_statement(query_for_c) == {A, B, A1}


def test_with_loader_criteria_simple_alias():
    aliased = sqlalchemy.orm.aliased(A)
    query_for_a = select(aliased).options(
        with_loader_criteria(A, A.id == 1, include_aliases=True),
    )

    assert all_entities_in_statement(query_for_a) == {A}
    # Crude way of detecting filter on a.id in the generated query.
    assert "a_1.id =" in str(query_for_a)


def test_with_loader_criteria_simple_subquery_no_alias():
    subquery = select(A).subquery(name="sub")
    query_for_a = select(subquery).options(
        with_loader_criteria(A, A.id == 1, include_aliases=True),
    )

    assert all_entities_in_statement(query_for_a) == {A}
    # Crude way of detecting filter on a.id in the generated query.
    assert "a.id =" in str(query_for_a)


@pytest.fixture
def test_oso():
    oso = Oso()
    oso.register_class(A)
    oso.register_class(B)

    # Allow 1.
    oso.load_str("allow(_, _, a: A) if a.id = 0; allow(_, _, b: B) if b.id = 0;")

    return oso


@pytest.fixture
def authorized_session(engine, test_oso):
    session = AuthorizedSession(
        bind=engine, oso=test_oso, user="u", checked_permissions={A: "a", B: "a"}
    )
    with session.begin():
        yield session


@pytest.mark.parametrize(
    "query_options",
    (
        (),
        (joinedload(A.bs),),
        (subqueryload(A.bs),),
        (selectinload(A.bs),),
    ),
)
def test_loads_relationship(test_data, authorized_session, query_options):
    """Confirm that relation is properly filtered.

    The policy (see fixture ``test_oso``) only allows one B, but there are two Bs
    on A with Id 0.

    We confirm that only 1 is returned to ensure the policy works properly.
    """
    a = authorized_session.query(A).options(*query_options).one()
    bs = a.bs
    assert a.id == 0
    assert len(bs) == 1
    assert bs[0].id == 0


def test_loads_relationship_no_auth(test_data, engine):
    """Sanity test that ``test_loads_relationship`` is actually testing authorization."""
    with Session(bind=engine) as s, s.begin():
        a = s.query(A).get(0)
        assert a.id == 0
        assert len(a.bs) == 2
