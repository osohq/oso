from pathlib import Path
import importlib
import sys

import pytest

from oso import Oso


def load_python(filename):
    """Load a python file into the knowledge base.

    Imports the file, and calls the load function from the file
    with the knowledge base.
    """
    filename = Path(__file__).parent / filename

    module_name_tail = Path(filename).stem
    module_name = f"polar.user.loaded.{module_name_tail}"
    spec = importlib.util.spec_from_file_location(module_name, filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)

    return module


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load(Path(__file__).parent / policy)

    return load


@pytest.mark.parametrize(
    "policy", ["01-simple.polar", "02-rbac.polar", "03-hierarchy.polar"],
)
def test_parses(oso, policy, load):
    # Test that policy parses and inline tests pass.
    load(policy)
    oso._kb_load()


def test_simple_01(oso, load):
    load("01-simple.polar")
    polar_classes = load_python("01-simple.py")
    Expense = polar_classes.Expense

    class User:
        def __init__(self, name):
            self.name = name

    oso.register_python_class(User)

    assert oso.allow(User("sam"), "view", Expense(0, submitted_by="sam"))
    assert not oso.allow(User("sam"), "view", Expense(0, submitted_by="steve"))


@pytest.mark.skip
def test_rbac_02(oso, load):
    load("02-rbac.polar")
    polar_classes = load_python("01-simple.py")

    class User:
        def __init__(self, name):
            self.name = name

    oso.register_python_class(User)

    # this needs more testing still, but the abac policies are not complete.
    # we either need to include the rbac example in this test or make the
    # policies include all required rules to execute.
