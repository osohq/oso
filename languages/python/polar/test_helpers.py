"""Set of test helpers to match test helpers from Python Polar."""
from pathlib import Path

import pytest

from polar.api import Polar

# DEFINED So pytests have same interface.
@pytest.fixture
def db():
    """ Set up the polar database """
    raise NotImplementedError()


@pytest.fixture
def polar():
    """ Set up a polar instance and tear it down after the test."""
    p = Polar()
    yield p
    del p


@pytest.fixture
def tell(polar):
    """ Define a fact or rule in the polar database """

    def _tell(f):
        # TODO (dhatch): Temporary until rewritten parser supports optional
        # semicolon.
        if not f.endswith(";"):
            f += ";"

        polar.load_str(f)

    return _tell


@pytest.fixture
def load_file(polar):
    """ Load a source file """

    def _load_file(f):
        path = Path(__file__).parent / f
        polar.load(path)

    return _load_file


@pytest.fixture
def query(polar):
    """ Query something and return the results as a list """

    def _query(q):
        return list(polar.query_str(q))

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
