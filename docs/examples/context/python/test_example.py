from pathlib import Path

import pytest

from oso import Oso
import os
from runpy import run_path

run_path("02-context.py")


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent.parent / policy)

    return load


def test_policy(oso, load):
    load("01-context.polar")

    os.environ["ENV"] = "production"
    assert not oso.allow("steve", "test", "policy")
    os.environ["ENV"] = "development"
    assert oso.allow("steve", "test", "policy")
