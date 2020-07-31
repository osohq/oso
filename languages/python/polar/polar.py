"""Communicate with the Polar virtual machine: load rules, make queries, etc."""

from datetime import datetime, timedelta
from pathlib import Path
from pprint import pprint

from _polar_lib import lib

from .errors import get_error
from .exceptions import PolarApiException, PolarRuntimeException
from .ffi import ffi_serialize, load_str, check_result, is_null, to_c_str, Predicate
from .host import Host
from .query import Query, QueryResult


CLASSES = {}
CONSTRUCTORS = {}


class Polar:
    """Polar API"""

    def __init__(self, classes=CLASSES, constructors=CONSTRUCTORS):
        self.ffi_polar = lib.polar_new()
        self.host = Host(self.ffi_polar)
        self.load_queue = []

        # Register built-in classes.
        self.register_class(datetime, name="Datetime")
        self.register_class(timedelta, name="Timedelta")

        # Pre-registered classes.
        for name, cls in classes.items():
            self.register_class(cls, name=name, from_polar=constructors[name])

    def __del__(self):
        del self.host
        lib.polar_free(self.ffi_polar)

    def clear(self):
        self.load_queue = []
        lib.polar_free(self.ffi_polar)
        self.ffi_polar = lib.polar_new()

    def load_file(self, policy_file):
        """Load in polar policies. By default, defers loading of knowledge base
        until a query is made."""
        policy_file = Path(policy_file)
        extension = policy_file.suffix
        if extension not in (".pol", ".polar"):
            raise PolarApiException(f"Polar files must have .pol or .polar extension.")
        if not policy_file.exists():
            raise PolarApiException(f"Could not find file: {policy_file}")
        if policy_file not in self.load_queue:
            self.load_queue.append(policy_file)

    def load_str(self, string):
        """Load a Polar string, checking that all inline queries succeed."""
        load_str(self.ffi_polar, string, None)

        # check inline queries
        while True:
            query = lib.polar_next_inline_query(self.ffi_polar, 0)
            if is_null(query):  # Load is done
                break
            else:
                try:
                    next(Query(query, host=self.host.copy()).run())
                except StopIteration:
                    raise PolarRuntimeException("Inline query in file failed.")

    def query(self, query):
        """Query for a predicate, parsing it if necessary.

        :param query: The predicate to query for.
        :param single: Whether to stop after the first result.

        :return: The result of the query.
        """
        self._load_queued_files()

        host = self.host.copy()
        if isinstance(query, str):
            query = check_result(
                lib.polar_new_query(self.ffi_polar, to_c_str(query), 0)
            )
        elif isinstance(query, Predicate):
            query = check_result(
                lib.polar_new_query_from_term(
                    self.ffi_polar, ffi_serialize(host.to_polar_term(query)), 0
                )
            )
        else:
            raise PolarApiException(f"Can not query for {query}")

        for res in Query(query, host=host).run():
            yield res["bindings"]

    def query_rule(self, name, *args):
        """Query for rule with name ``name`` and arguments ``args``.

        :param name: The name of the predicate to query.
        :param args: Arguments for the predicate.

        :return: The result of the query.
        """
        return self.query(Predicate(name=name, args=args))

    def repl(self, load=True):
        """Start an interactive REPL session."""
        if load:
            import sys

            for f in sys.argv[1:]:
                self.load_file(f)
        self._load_queued_files()

        while True:
            try:
                query = input("query> ").strip(";")
            except EOFError:
                return
            ffi_query = lib.polar_new_query(self.ffi_polar, to_c_str(query), 0)
            if is_null(ffi_query):
                print("Parse error: ", get_error())
                continue

            result = False
            try:
                query = Query(ffi_query, host=self.host.copy()).run()
                for res in query:
                    result = True
                    bindings = res["bindings"]
                    pprint(bindings if bindings else True)
            except PolarRuntimeException as e:
                pprint(e)
                continue
            if not result:
                pprint(False)

    def register_class(self, cls, *, name=None, from_polar=None):
        """Register `cls` as a class accessible by Polar. `from_polar` can
        either be a method or a string. In the case of a string, Polar will
        look for the method using `getattr(cls, from_polar)`."""
        cls_name = self.host.cache_class(cls, name, from_polar)
        self.register_constant(cls_name, cls)

    def register_constant(self, name, value):
        """Register `value` as a Polar constant variable called `name`."""
        name = to_c_str(name)
        value = ffi_serialize(self.host.to_polar_term(value))
        lib.polar_register_constant(self.ffi_polar, name, value)

    def _load_queued_files(self):
        """Load queued policy files into the knowledge base."""
        while self.load_queue:
            filename = self.load_queue.pop(0)
            with open(filename) as file:
                load_str(self.ffi_polar, file.read(), filename)


def polar_class(_cls=None, *, name=None, from_polar=None):
    """Decorator to register a Python class with Polar.
    An alternative to ``register_class()``.

    :param str from_polar: Name of class function to create a new instance from ``fields``.
                           Defaults to class constructor.
    """

    def wrap(cls):
        cls_name = cls.__name__ if name is None else name
        CLASSES[cls_name] = cls
        CONSTRUCTORS[cls_name] = from_polar or cls
        return cls

    if _cls is None:
        return wrap

    return wrap(_cls)
