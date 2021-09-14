import pytest

from .app import oso, Organization, Repository, User


@pytest.fixture
def test_oso():
    oso.clear_rules()
    yield oso


@pytest.fixture
def alpha_association():
    yield Organization("Alpha Association")


@pytest.fixture
def beta_business():
    yield Organization("Beta Business")


@pytest.fixture
def affine_types(alpha_association):
    yield Repository("Affine Types", alpha_association)


@pytest.fixture
def allocator(alpha_association):
    yield Repository("Allocator", alpha_association)


@pytest.fixture
def bubble_sort(beta_business):
    yield Repository("Bubble Sort", beta_business)


@pytest.fixture
def benchmarks(beta_business):
    yield Repository("Benchmarks", beta_business)


@pytest.fixture
def ariana(alpha_association):
    ariana = User("Ariana", set())
    ariana.assign_role_for_resource("owner", alpha_association)
    yield ariana


@pytest.fixture
def bhavik(bubble_sort, benchmarks):
    bhavik = User("Bhavik", set())
    bhavik.assign_role_for_resource("contributor", bubble_sort)
    bhavik.assign_role_for_resource("maintainer", benchmarks)
    yield bhavik
