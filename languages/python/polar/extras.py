import re

from typing import List
from datetime import datetime, timedelta


class Http:
    """A resource accessed via HTTP."""

    def __init__(self, path="", query={}, hostname=None):
        self.path = path
        self.query = query
        self.hostname = hostname

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
            yield match.groupdict()


class Datetime(datetime):
    """ Polar wrapper for Python `datetime`. Allows Datetime to be created with no arguments by providing
    an arbitrary default. This is a workaround for lack of class method support, allowing `now()` to be
    called on a default Datetime instance, e.g. `x = Datetime{}.now`. Also improves the syntax for
    subtraction. """

    def __new__(
        cls, year=1970, month=1, day=1, hour=0, minute=0, second=0, microsecond=0
    ):
        return super().__new__(cls, year, month, day, hour, minute, second, microsecond)

    def from_datetime(dt):
        return Datetime(
            year=dt.year,
            month=dt.month,
            day=dt.day,
            hour=dt.hour,
            minute=dt.minute,
            second=dt.second,
            microsecond=dt.microsecond,
        )

    def now(self):
        return Datetime.from_datetime(datetime.now())

    def sub(self, other):
        return self.__sub__(other)


class Timedelta(timedelta):
    """ Polar wrapper for Python `timedelta`. Not really a purpose to this other than consistency
    with the Polar class naming convention given that the Polar `Datetime` class is capitalized. """

    def __new__(
        cls,
        days=0,
        seconds=0,
        microseconds=0,
        milliseconds=0,
        minutes=0,
        hours=0,
        weeks=0,
    ):
        return super().__new__(
            cls, days, seconds, microseconds, milliseconds, minutes, hours, weeks
        )
