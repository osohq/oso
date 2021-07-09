import json
import tempfile

import pytest

from oso import Oso, Variable, Predicate



def test_trace():
    oso = Oso()
    oso.load_str("f(1); f(2);")

    with tempfile.NamedTemporaryFile(suffix=".polar") as f:
        f.write("""
            f(x, y) if x > 0 and y < 1 and x < 5;
            f(x, _) if x = 1;
        """.encode("ascii"))
        f.flush()

        oso.load_file(f.name)
        query = oso._query(Predicate("f", (1, 0)))

        results = [r for r in query.run()]

        trace = query.trace()
        with open("trace.json", "w") as fw:
            json.dump(trace, fw)