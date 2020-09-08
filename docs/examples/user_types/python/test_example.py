from pathlib import Path
import pytest

from oso import Oso

# This example is pseudo-code mostly, queries a nonexistent database.
# This test shims the classes and makes sure the polar parses.


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent.parent / policy)

    return load


def test_parses(oso, load):
    class InternalUser:
        pass

    class Customer:
        pass

    class AccountManager:
        pass

    class AccountData:
        pass

    oso.register_class(InternalUser)
    oso.register_class(Customer)
    oso.register_class(AccountManager)
    oso.register_class(AccountData)

    # Test that policy parses and inline tests pass.
    load("user_policy.polar")
