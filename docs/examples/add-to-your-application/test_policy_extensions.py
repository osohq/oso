from oso import Oso


def test_parses():
    oso = Oso()
    oso.register_class(type("User", (), {}))
    oso.register_class(type("Repository", (), {}))
    oso.load_files(['write-rules.polar'])
