from pathlib import Path

import pytest

from oso import Oso
import imp
import os


@pytest.fixture
def oso():
    return Oso()


def test_policy(oso):
    print(os.curdir)
    oso.load_file("../../docs/examples/context/01-context.polar")
    imp.load_source("context", "../../docs/examples/context/python/02-context.py")

    oso._load_queued_files()

    os.environ["ENV"] = "production"
    assert not oso.allow("steve", "test", "policy")
    os.environ["ENV"] = "development"
    assert oso.allow("steve", "test", "policy")
