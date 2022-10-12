import io
import os
import re

import pytest

from polar.exceptions import PolarRuntimeError

FOO_REPR = "SPECIAL FOO REPR"


class Foo:
    def __repr__(self) -> str:
        return FOO_REPR


def test_repr_when_logging(polar, capsys):
    old_polar_log = os.getenv("POLAR_LOG")
    os.environ["POLAR_LOG"] = "1"
    polar.register_class(Foo)
    polar.load_str("f(_foo: Foo) if 1 = 1;")
    list(polar.query_rule("f", Foo()))
    captured = capsys.readouterr()
    assert f"QUERY RULE: f({FOO_REPR} TYPE `Foo`)" in captured.out
    if not old_polar_log:
        os.unsetenv("POLAR_LOG")


def test_repr_in_error(polar):
    polar.register_class(Foo)
    # This will throw an error because foo.hello is not allowed
    polar.load_str("f(foo: Foo) if foo.hello;")
    with pytest.raises(PolarRuntimeError) as excinfo:
        list(polar.query_rule("f", Foo()))
    assert f"f({FOO_REPR} TYPE `Foo`)" in excinfo.value.stack_trace


def test_repr_when_debugging(polar, monkeypatch, capsys):
    polar.register_class(Foo)
    polar.load_str("f(_foo: Foo) if debug() and 1 = 1;")
    monkeypatch.setattr("sys.stdin", io.StringIO("bindings"))
    list(polar.query_rule("f", Foo()))
    captured = capsys.readouterr()
    assert re.search(rf"_foo_[0-9]+ = {FOO_REPR} TYPE `Foo`", captured.out)
