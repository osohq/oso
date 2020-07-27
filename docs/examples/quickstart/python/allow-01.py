from oso import Oso


def setup_oso():
    oso = Oso()
    return oso


oso = setup_oso()
oso = Oso()
actor = "alice@example.com"
resource = EXPENSES[1]
oso.is_allowed(actor, "GET", resource)
