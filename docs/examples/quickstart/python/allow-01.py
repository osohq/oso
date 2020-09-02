from oso import Oso
from expense import EXPENSES


def setup_oso():
    oso = Oso()
    return oso


oso = setup_oso()
oso = Oso()
actor = "alice@example.com"
resource = EXPENSES[1]
oso.is_allowed(actor, "GET", resource)
