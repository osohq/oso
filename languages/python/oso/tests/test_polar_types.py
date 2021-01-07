from polar.polar_types import Call, Value, deserialize_json, serialize_json
import re


def test_deserialize():
    term_json = """{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
    }"""

    res = deserialize_json(term_json, Value)
    expected = Call(name="foo", args=[0], kwargs={"bar": 1})
    assert res == expected

    res = serialize_json(res, Value)
    regex = re.compile(r"[ \n]")
    # compare ignoring whitespace
    assert regex.sub("", res) == regex.sub("", term_json)
