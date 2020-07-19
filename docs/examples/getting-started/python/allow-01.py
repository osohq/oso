from oso import Oso


def setup_oso():
    oso = Oso()
    return oso


oso = setup_oso()


oso.allow("alice", "view", "expense")
