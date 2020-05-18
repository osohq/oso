from pathlib import Path

from polar import Polar
from polar.test_helpers import db, polar, tell, load_file, query, qeval, qvar

import pytest


def test_anything_works():
    p = Polar()
    p.load_str("f(1);")
    results = list(p.query_str("f(x)"))
    assert results[0]["x"] == 1
    results = list(p.query_str("f(y)"))
    assert results[0]["y"] == 1
    del p


def test_helpers(polar, load_file, query, qeval, qvar):
    load_file(Path(__file__).parent / "test_file.polar")  # f(1);
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]


def test_data_conversions(polar, qvar):
    polar.load_str('a(1);b("two");c(true);d([1,"two",true]);')
    assert qvar("a(x)", "x", one=True) == 1
    assert qvar("b(x)", "x", one=True) == "two"
    assert qvar("c(x)", "x", one=True)
    assert qvar("d(x)", "x", one=True) == [1, "two", True]


def test_external(polar, qvar):
    assert qvar("Foo{start: 100}.call_me(105) = x", "x") == [100, 101, 102, 103, 104]
