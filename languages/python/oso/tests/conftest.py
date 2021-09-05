"""Set of test helpers to match test helpers from Python Polar."""
import pytest
from typing import Dict

from polar import Polar


# DEFINED So pytests have same interface.
@pytest.fixture
def db():
    """Set up the polar database"""
    raise NotImplementedError()


TEST_CLASSES: Dict[str, type] = {}


@pytest.fixture
def polar():
    """Set up a polar instance and tear it down after the test."""
    p = Polar()
    yield p
    del p


@pytest.fixture
def query(polar):
    """Query something and return the results as a list"""

    def _query(q):
        return list(r["bindings"] for r in polar.query(q))

    return _query


@pytest.fixture
def qeval(query):
    """Query something and return if there's exactly 1 result"""

    def _qeval(q):
        result = list(query(q))
        return len(result) == 1

    return _qeval


@pytest.fixture
def is_allowed(polar):
    """Check if actor may perform action on resource."""

    def _is_allowed(actor, action, resource):
        return len(list(polar.query_rule("allow", actor, action, resource))) > 0

    return _is_allowed


@pytest.fixture
def qvar(query):
    """Query something and pull out the results for the variable v"""

    def _qvar(q, v, one=False):
        results = query(q)
        if one:
            assert len(results) == 1, "expected one result"
            return results[0][v]
        return [env[v] for env in results]

    return _qvar
