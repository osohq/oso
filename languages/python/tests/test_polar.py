from pathlib import Path

from polar import Polar

import pytest

def test_anything_works():
    p = Polar()
    p.load_str("f(1);")
    results = list(p.query("f(x)"))
    assert results[0]["x"] == 1
    results = list(p.query("f(y)"))
    assert results[0]["y"] == 1
    del p

@pytest.fixture
def polar():
    """ Set up a polar instance and tear it down after the test."""
    p = Polar()
    yield p
    del p

@pytest.fixture
def load_file(polar):
    """ Load a source file """
    def _load_file(f):
        path = Path(__file__).parent / f
        with open(path, 'r') as f:
            data = f.read()
        polar.load_str(data)

    return _load_file

@pytest.fixture
def query(polar):
    """ Query something and return the results as a list """
    def _query(q):
        return list(polar.query(q))

    return _query

@pytest.fixture
def qeval(polar, query):
    """ Query something and return if there's exactly 1 result """
    def _qeval(q):
        result = query(q)
        return len(result) == 1

    return _qeval

@pytest.fixture
def qvar(polar, query):
    """ Query something and pull out the results for the variable v """
    def _qvar(q, v, one=False):
        results = query(q)
        if one:
            if len(results) == 1:
                return results[0][v]
            else:
                return None
        return [env[v] for env in results]

    return _qvar

def test_helpers(polar, load_file, query, qeval, qvar):
    load_file("test_file.polar") # f(1);
    assert query("f(x)") == [{'x': 1}, {'x': 2}, {'x': 3}]
    assert qvar("f(x)", "x") == [1,2,3]
