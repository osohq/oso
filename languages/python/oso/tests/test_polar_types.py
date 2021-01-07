from polar.polar_types import Call, Value, deserialize_json, ValueNumber, NumericInteger


def test_deserialize():
    term_json = """{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
    }"""

    res = deserialize_json(term_json, Value)
    expected = Call(name="foo", args=[ValueNumber(0)], kwargs={"bar": ValueNumber(1)})
    assert res == expected
