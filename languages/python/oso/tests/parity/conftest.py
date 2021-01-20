# oso parity test runner

import pdb
import pytest
import re
import sys, inspect
from pathlib import Path
from collections import OrderedDict

import yaml  # we need a yaml parser, e.g. PyYAML
from oso import Oso, Variable

# Load in all the required application classes
from . import classes
from .constants import CONSTANTS

def pytest_collect_file(parent, path):
    if path.ext == ".yml":
        return YamlFile(path, parent)


class YamlFile(pytest.File):
    def collect(self):
        raw = yaml.safe_load(self.fspath.open())
        name = raw.get("name")
        description = raw.get("description", "")
        policies = raw.get("policies", [])
        for case in raw["cases"]:
            yield TestCase(self, name, description, policies, case)

class Result(OrderedDict):
    """Result class provides helper methods for comparing results"""

    def __init__(self, d):
        def to_result(v):
            if isinstance(v, dict):
                return Result(v)
            else:
                return v
        super().__init__({k: to_result(v) for k, v in d.items()})
    
    def __eq__(self, other):
        if "repr" in self:
            return self["repr"] == repr(other)
        else:
            return super().__eq__(other)
    
    def __repr__(self):
        return repr(dict(self))

def to_input(v):
    if isinstance(v, dict):
        if "type" in v:
            cls = getattr(classes, v["type"])
            args = [to_input(v) for v in v.get("args", [])]
            kwargs = {k: to_input(v) for k, v in v.get("kwargs", {}).items()}
            return cls(*args, **kwargs)
        elif "var" in v:
            return Variable(v["var"])
    return v

class TestCase(pytest.Item):
    def __init__(
        self,
        parent: YamlFile,
        name: str,
        description: str,
        policies: list,
        case: dict,
    ):
        super().__init__(name, parent)
        self.name = name
        self.policies = policies
        self.description = description
        self.case = case
        oso = Oso()
        for _, c in inspect.getmembers(classes):
            if isinstance(c, type):
                oso.register_class(c)
        for k, v in CONSTANTS.items():
            oso.register_constant(v, k)
        for policy in self.policies:
            path = Path(__file__).parent.resolve()
            oso.load_file(f"{path}/policy/{policy}.polar")
        self.oso = oso

    def runtest(self):
        query = self.case.get("query", None)
        load = self.case.get("load", None)
        inputs = self.case.get("input", None)
        if inputs:
            inputs = [to_input(v) for v in inputs]
            test_query = self.oso.query_rule(query, *inputs)
        else:
            test_query = self.oso.query(query)
        expected_result = [Result(res) for res in self.case.get("result", [{}])]
        try:
            if load:
                self.oso.load_str(load)
            result = [res.get("bindings", {}) for res in test_query]
        except Exception as e:
            if "err" in self.case:
                expected_err = self.case["err"]
                if re.search(expected_err, str(e)):
                    return
                else:
                    raise YamlException(self, query, inputs, e, expected_err)
            raise YamlException(self, query, inputs, e, expected_result)
        if result != expected_result:
            raise YamlException(self, query, inputs, result, expected_result)

    def repr_failure(self, excinfo):
        """ called when self.runtest() raises an exception. """
        if isinstance(excinfo.value, YamlException):
            test_case, query, inputs, result, expected = excinfo.value.args
            if inputs:
                query = f'{query}({",".join(repr(x) for x in inputs)})'
            return "\n".join(
                [
                    "usecase execution failed",
                    f"   spec failed: {query}",
                    f"   expected: {expected}",
                    f"   got: {repr(result)}",
                ]
            )
        else:
            return f"{excinfo}"

    def reportinfo(self):
        return (
            self.fspath,
            0,
            "%s %s" % (self.description, self.case.get("description", "")),
        )


class YamlException(Exception):
    """ custom exception for error reporting. """
