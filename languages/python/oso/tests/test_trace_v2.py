import json

import pytest

from oso import Oso, Variable, Predicate

def test_trace():
    oso = Oso()
    oso.load_str("f(1); f(2);")
    query = oso._query(Predicate("f", (Variable("x"),)))

    results = [r for r in query.run()]
    trace = json.loads(query.trace())
    breakpoint()
