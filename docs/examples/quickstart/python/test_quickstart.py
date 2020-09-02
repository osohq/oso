from pathlib import Path

import pytest

from runpy import run_path

run_path("allow-01.py")

from expense import *
from oso import Oso

import server


def test_quickstart_policy_2():
    oso = Oso()
    alice = "alice@example.com"
    expense = EXPENSES[1]
    assert not oso.is_allowed(alice, "GET", expense)
    oso.register_class(Expense)
    oso.load_file("../polar/expenses-02.polar")
    assert oso.is_allowed(alice, "GET", expense)
    assert not oso.is_allowed("bhavik@example.com", "GET", expense)


def test_quickstart_policy_3():
    oso = Oso()
    oso.register_class(Expense)
    oso.load_file("../polar/expenses-03-py.polar")
    expense = EXPENSES[1]
    assert oso.is_allowed("bhavik@example.com", "GET", expense)
    assert not oso.is_allowed("bhavik@foo.com", "GET", expense)


def test_quickstart_policy_4():
    oso = Oso()
    oso.register_class(Expense)
    oso.load_file("../polar/expenses-04.polar")
    assert oso.is_allowed("alice@example.com", "GET", EXPENSES[1])
    assert not oso.is_allowed("alice@example.com", "GET", EXPENSES[3])
    assert not oso.is_allowed("bhavik@example.com", "GET", EXPENSES[1])
    assert oso.is_allowed("bhavik@example.com", "GET", EXPENSES[3])