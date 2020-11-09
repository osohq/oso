import re

from typing import List


class Http:
    """A resource accessed via HTTP."""

    def __init__(self, hostname="", path="", query={}):
        self.hostname = hostname
        self.path = path
        self.query = query

    def __repr__(self):
        return str(self)

    def __str__(self):
        q = {k: v for k, v in self.query.items()}
        host_str = f'hostname="{self.hostname}"' if self.hostname else None
        path_str = f'path="{self.path}"' if self.path != "" else None
        query_str = f"query={q}" if q != {} else None
        field_str = ", ".join(x for x in [host_str, path_str, query_str] if x)
        return f"Http({field_str})"


class PathMapper:
    """Map from a template string with capture groups of the form
    ``{name}`` to a dictionary of the form ``{name: captured_value}``

    :param template: the template string to match against
    """

    def __init__(self, template):
        capture_group = re.compile(r"({([^}]+)})")
        for outer, inner in capture_group.findall(template):
            if inner == "*":
                template = template.replace(outer, ".*")
            else:
                template = template.replace(outer, f"(?P<{inner}>[^/]+)")
        self.pattern = re.compile("^" + template + "$")

    def map(self, string):
        match = self.pattern.match(string)
        if match:
            return match.groupdict()
