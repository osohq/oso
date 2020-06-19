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
        oso.load_file(Path(__file__).parent / policy)
        oso._load_queued_files()

    return load


@pytest.mark.parametrize(
    "policy", ["01-simple.polar", "02-rbac.polar", "03-hierarchy.polar"],
)
def test_parses(oso, policy, load):
    polar_classes = load_python("01-simple.py")
    # Test that policy parses and inline tests pass.
    load(policy)
    oso._load_queued_files()


EXPENSES_DEFAULT = {
    "location": "NYC",
    "amount": 50,
    "project_id": 2,
}


def test_simple_01(oso, load):
    polar_classes = load_python("01-simple.py")
    User = polar_classes.User
    Expense = polar_classes.Expense
    load("01-simple.polar")

    assert oso.allow(
        User("sam"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="sam")
    )
    assert not oso.allow(
        User("sam"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="steve"),
    )


def test_rbac_02(oso, load):
    polar_classes = load_python("01-simple.py")
    User = polar_classes.User
    Expense = polar_classes.Expense
    load("02-rbac.polar")

    oso.load_str('role(User { name: "sam" }, "admin", Project { id: 2 });')
    expense = Expense(location="NYC", amount=50, project_id=0, submitted_by="steve")
    assert not oso.allow(User("sam"), "view", expense)
    expense = Expense(location="NYC", amount=50, project_id=2, submitted_by="steve")
    assert oso.allow(User("sam"), "view", expense)


def test_hierarchy_03(oso, load):
    polar_classes = load_python("01-simple.py")
    User = polar_classes.User
    Expense = polar_classes.Expense
    load("03-hierarchy.polar")

    assert oso.allow(
        User("bhavik"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="alice")
    )
    assert oso.allow(
        User("cora"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="alice"),
    )
    assert oso.allow(
        User("cora"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="bhavik"),
    )
    assert not oso.allow(
        User("bhavik"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="cora"),
    )
    assert not oso.allow(
        User("alice"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="cora"),
    )
    assert not oso.allow(
        User("alice"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="bhavik"),
    )
