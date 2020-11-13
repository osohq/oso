# oso parity test runner

import pdb
import pytest
import re
import sys, inspect

import yaml  # we need a yaml parser, e.g. PyYAML
from oso import Oso

# Load in all the required application classes
import classes


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


class TestCase(pytest.Item):
    def __init__(
        self, parent: YamlFile, name: str, description: str, policies: list, case: dict,
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
        for policy in self.policies:
            oso.load_file(f"policy/{policy}.polar")
        self.oso = oso

    def runtest(self):
        query = self.case["query"]
        expected_result = self.case.get("result", None)
        try:
            result = [res.get("bindings", {}) for res in self.oso.query(query)]
        except Exception as e:
            if "err" in self.case and re.search(self.case["err"], str(e)):
                return
            raise YamlException(self, query, e, expected_result)
        if result != expected_result:
            raise YamlException(self, query, result, expected_result)

        # if "many" in expected_result:
        #     if result != expected_result["many"]:
        # elif "one" in expected_result:
        #     if result != [expected_result["one"]]:
        #         raise YamlException(self, query, result, expected_result)
        # elif "none" in expected_result:
        #     if len(result) != 0:
        #         raise YamlException(self, query, result, expected_result)

    def repr_failure(self, excinfo):
        """ called when self.runtest() raises an exception. """
        if isinstance(excinfo.value, YamlException):
            test_case, query, result, expected = excinfo.value.args
            return "\n".join(
                [
                    "usecase execution failed",
                    f"   spec failed: {query}",
                    f"   expected: {expected}",
                    f"   got: {result}",
                ]
            )
        else:
            return f"{excinfo}"

    def reportinfo(self):
        return (
            self.fspath,
            0,
            "%s %s" % (self.description, self.case["description"]),
        )


class YamlException(Exception):
    """ custom exception for error reporting. """
