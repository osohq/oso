from oso import Oso
from company import Company, StartUp

# test-company-start
import pytest
from pathlib import Path


def test_policy():
    oso = Oso()
    fname = Path(__file__).parent / "company.polar"

    oso.load(fname)

    # Test Company resources
    assert oso.allow("leina", "read", Company(id=2))
    assert oso.allow("sam", "read", Company(id=1))

    assert not oso.allow("sam", "read", Company(id=2))
    assert not oso.allow("dhatch", "read", Company(id=1))
    # test-company-end

    # test-startup-start
    # Test StartUp resources
    assert oso.allow("leina", "read", StartUp(id=2))
    assert oso.allow("sam", "read", StartUp(id=1))

    assert not oso.allow("sam", "read", StartUp(id=2))
    assert not oso.allow("dhatch", "read", StartUp(id=1))

    assert oso.allow("chris", "read", StartUp(id=1))
    assert oso.allow("reid", "read", StartUp(id=2))
    # test-startup-end


def test_policy_with_cut():
    oso = Oso()
    oso.load(Path(__file__).parent / "company.polar")
    oso.load(Path(__file__).parent / "company_cut.polar")

    # Test Company resources
    assert oso.allow("leina", "read", Company(id=2))
    assert oso.allow("sam", "read", Company(id=1))

    assert not oso.allow("sam", "read", Company(id=2))
    assert not oso.allow("dhatch", "read", Company(id=1))

    # test-cut-start
    # Test StartUp resources
    assert not oso.allow("leina", "read", StartUp(id=2))
    assert not oso.allow("sam", "read", StartUp(id=1))

    assert not oso.allow("sam", "read", StartUp(id=2))
    assert not oso.allow("dhatch", "read", StartUp(id=1))

    assert oso.allow("chris", "read", StartUp(id=1))
    assert oso.allow("reid", "read", StartUp(id=2))


# test-cut-end
