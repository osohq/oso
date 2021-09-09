from pathlib import Path

import pytest

from oso import Oso

from . import inheritance_external


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def patient():
    return "Bob"


@pytest.fixture
def med_staff(patient):
    return inheritance_external.User(role="medical_staff", treated=[patient])


@pytest.fixture
def med_staff_bad_patient():
    return inheritance_external.User(role="medical_staff", treated=["Not Bob"])


@pytest.fixture
def reg_staff(patient):
    return inheritance_external.User(role="reg_staff", treated=[patient])


@pytest.fixture
def order(patient):
    return inheritance_external.Order(patient=patient)


@pytest.fixture
def lab(patient):
    return inheritance_external.Lab(patient=patient)


@pytest.fixture
def test(patient):
    return inheritance_external.Test(patient=patient)


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent.parent / policy)

    return load


@pytest.mark.parametrize(
    "policy",
    [
        "01-polar.polar",
        "02-nested-rule.polar",
        "03-specializer.polar",
        "04-one-specializer.polar",
        pytest.param("05-group.polar", marks=pytest.mark.xfail(reason="no groups")),
        "06-permissive-restrictive.polar",
        "07-common-cut.polar",
    ],
)
def test_loads(oso, policy, load):
    # Test that policy loads.
    oso.load_file(Path(__file__).parent.parent / policy)


@pytest.mark.parametrize(
    "policy",
    [
        "01-polar.polar",
        "02-nested-rule.polar",
        "03-specializer.polar",
        "04-one-specializer.polar",
        # Note this one isn't that meaningful because we use the same
        # externals that do have inheritance
        pytest.param("05-group.polar", marks=pytest.mark.xfail(reason="no groups")),
    ],
)
def test_rule_for_med_staff(oso, load, policy, med_staff, order, lab, test):
    """Test that rule matches for medical staff."""
    load(policy)
    assert oso.is_allowed(med_staff, "read", order)
    assert oso.is_allowed(med_staff, "read", lab)
    assert oso.is_allowed(med_staff, "read", test)


@pytest.mark.parametrize(
    "policy",
    [
        "01-polar.polar",
        "02-nested-rule.polar",
        "03-specializer.polar",
        "04-one-specializer.polar",
        # Note this one isn't that meaningful because we use the same
        # externals that do have inheritance
        pytest.param("05-group.polar", marks=pytest.mark.xfail(reason="no groups")),
    ],
)
def test_rule_for_med_staff_bad_patient(
    oso, load, policy, med_staff_bad_patient, order, lab, test
):
    """Test that rule doesn't match for medical staff that did not treat the
    same patient as the resource."""
    load(policy)
    assert not oso.is_allowed(med_staff_bad_patient, "read", order)
    assert not oso.is_allowed(med_staff_bad_patient, "read", lab)
    assert not oso.is_allowed(med_staff_bad_patient, "read", test)


@pytest.mark.parametrize(
    "policy",
    [
        "01-polar.polar",
        "02-nested-rule.polar",
        "03-specializer.polar",
        "04-one-specializer.polar",
        # Note this one isn't that meaningful because we use the same
        # externals that do have inheritance
        pytest.param("05-group.polar", marks=pytest.mark.xfail(reason="no groups")),
    ],
)
def test_rule_for_reg_staff(oso, load, policy, reg_staff, order, lab, test):
    """Test that rule doesn't match for not medical staff."""
    load(policy)
    assert not oso.is_allowed(reg_staff, "read", order)
    assert not oso.is_allowed(reg_staff, "read", lab)
    assert not oso.is_allowed(reg_staff, "read", test)
