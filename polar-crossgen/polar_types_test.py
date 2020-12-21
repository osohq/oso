from polar_types import Value, Call, Value__Call, Value__Number, Numeric__Integer
from serde_json import deserialize_json


def test_deserialize():
    term_json = """{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
    }"""

    assert deserialize_json(term_json, Value) == Value__Call(value=Call(
        name="foo",
        args=[Value__Number(value=Numeric__Integer(value=0))],
        kwargs={"bar": Value__Number(value=Numeric__Integer(value=1))},
    ))
