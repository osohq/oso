from pathlib import Path
from runpy import run_path
import sys

import pytest

from oso import Oso

polar_classes = run_path("01-simple.py")
User = polar_classes["User"]
Expense = polar_classes["Expense"]


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent.parent / policy)

    return load


@pytest.mark.parametrize(
    "policy", ["01-simple.polar", "02-rbac.polar", "03-hierarchy.polar"]
)
def test_parses(oso, policy, load):
    """Test that policy parses and inline tests pass."""
    load(policy)


EXPENSES_DEFAULT = {"location": "NYC", "amount": 50, "project_id": 2}


def test_simple_01(oso, load):
    load("01-simple.polar")

    assert oso.is_allowed(
        User("sam"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="sam")
    )
    assert not oso.is_allowed(
        User("sam"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="steve")
    )


def test_rbac_02(oso, load):
    load("02-rbac.polar")

    oso.load_str('role(_: User { name: "sam" }, "admin", __: Project { id: 2 });')
    expense = Expense(location="NYC", amount=50, project_id=0, submitted_by="steve")
    assert not oso.is_allowed(User("sam"), "view", expense)
    expense = Expense(location="NYC", amount=50, project_id=2, submitted_by="steve")
    assert oso.is_allowed(User("sam"), "view", expense)


def test_hierarchy_03(oso, load):
    load("03-hierarchy.polar")

    assert oso.is_allowed(
        User("bhavik"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="alice")
    )
    assert oso.is_allowed(
        User("cora"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="alice")
    )
    assert oso.is_allowed(
        User("cora"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="bhavik")
    )
    assert not oso.is_allowed(
        User("bhavik"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="cora")
    )
    assert not oso.is_allowed(
        User("alice"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="cora")
    )
    assert not oso.is_allowed(
        User("alice"), "view", Expense(**EXPENSES_DEFAULT, submitted_by="bhavik")
    )
