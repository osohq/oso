import pytest
from oso import Oso, Relation
from sqlalchemy import create_engine
from sqlalchemy.types import String, Boolean
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base
from helpers import check_authz

Base = declarative_base()


class Bar(Base):  # type: ignore
    __tablename__ = "bars"

    id = Column(String(), primary_key=True)
    is_cool = Column(Boolean())
    is_still_cool = Column(Boolean())


class Foo(Base):  # type: ignore
    __tablename__ = "foos"

    id = Column(String(), primary_key=True)
    bar_id = Column(String, ForeignKey("bars.id"))
    is_fooey = Column(Boolean())


engine = create_engine("sqlite:///:memory:")
Base.metadata.create_all(engine)
Session = sessionmaker(bind=engine)
session = Session()

hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)
something_foo = Foo(id="something", bar_id="hello", is_fooey=False)
another_foo = Foo(id="another", bar_id="hello", is_fooey=True)
third_foo = Foo(id="third", bar_id="hello", is_fooey=True)
fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True)

for obj in [
    hello_bar,
    goodbye_bar,
    hershey_bar,
    something_foo,
    another_foo,
    third_foo,
    fourth_foo,
]:
    session.add(obj)
    session.commit()


@pytest.fixture
def oso():
    oso = Oso()

    oso.set_data_filtering_query_defaults(
        exec_query=lambda query: query.all(), combine_query=lambda q1, q2: q1.union(q2)
    )

    # @TODO: Somehow the session needs to get in here, didn't think about that yet... Just hack for now and use a global
    # one.
    def get_bars(constraints):
        query = session.query(Bar)
        for constraint in constraints:
            field = getattr(Bar, constraint.field)
            if constraint.kind == "Eq":
                query = query.filter(field == constraint.value)
            elif constraint.kind == "Neq":
                query = query.filter(field != constraint.value)
            elif constraint.kind == "In":
                query = query.filter(field.in_(constraint.value))
            # ...
        return query

    oso.register_class(
        Bar,
        fields={"id": str, "is_cool": bool, "is_still_cool": bool},
        build_query=get_bars,
    )

    def get_foos(constraints):
        query = session.query(Foo)
        for constraint in constraints:
            field = getattr(Foo, constraint.field)
            if constraint.kind == "Eq":
                query = query.filter(field == constraint.value)
            elif constraint.kind == "Neq":
                query = query.filter(field != constraint.value)
            elif constraint.kind == "In":
                query = query.filter(field.in_(constraint.value))
            # ...
        return query

    oso.register_class(
        Foo,
        fields={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "bar": Relation(
                kind="one", other_type="Bar", my_field="bar_id", other_field="id"
            ),
        },
        build_query=get_foos,
    )

    return oso


def test_sqlalchemy_relationship(oso):
    oso.load_str(
        """
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.is_cool = true and
            resource.is_fooey = true;
    """
    )
    assert oso.is_allowed("steve", "get", another_foo)

    results = oso.authorized_resources("steve", "get", Foo)
    assert len(results) == 2


def test_sqlalchemy_neq(oso):
    oso.load_str(
        """
        allow("steve", "get", foo: Foo) if foo.bar.id != "hello";
        allow("steve", "put", foo: Foo) if foo.bar.id != "goodbye";
    """
    )
    check_authz(oso, "steve", "get", Foo, [fourth_foo])
    check_authz(
        oso,
        "steve",
        "put",
        Foo,
        [another_foo, third_foo, something_foo],
    )
