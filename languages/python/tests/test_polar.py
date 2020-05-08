from polar import Polar

import pytest

def test_anything_works():
    p = Polar()
    p.load_str("f(1);")
    results = list(p.query("f(x)"))
    assert results[0]["x"] == 1