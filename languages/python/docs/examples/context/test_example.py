from pathlib import Path

import pytest

from oso import Oso
import imp
import os


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent / policy)

    return load


def test_policy(oso, load):
    load("01-context.polar")
    imp.load_source("context", "02-context.py")

    oso._load_queued_files()

    os.environ["ENV"] = "production"
    assert not oso.allow("steve", "test", "policy")
    os.environ["ENV"] = "development"
    assert oso.allow("steve", "test", "policy")
