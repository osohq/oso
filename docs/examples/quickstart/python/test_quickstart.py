from pathlib import Path

import pytest

from runpy import run_path

run_path("allow-01.py")

from expense import *
from oso import Oso


def test_quickstart_policy_1():
    oso = Oso()
    oso.register_class(Expense)
    oso.load_file("../polar/expenses-01-python.polar")
    expense = EXPENSES[1]
    assert oso.is_allowed("bhavik@example.com", "GET", expense)
    assert not oso.is_allowed("bhavik@foo.com", "GET", expense)


def test_quickstart_policy_4():
    oso = Oso()
    oso.register_class(Expense)
    oso.load_file("../polar/expenses-02-python.polar")
    assert oso.is_allowed("alice@example.com", "GET", EXPENSES[1])
    assert not oso.is_allowed("alice@example.com", "GET", EXPENSES[3])
    assert not oso.is_allowed("bhavik@example.com", "GET", EXPENSES[1])
    assert oso.is_allowed("bhavik@example.com", "GET", EXPENSES[3])
