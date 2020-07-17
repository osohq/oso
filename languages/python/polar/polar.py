"""Communicate with the Polar virtual machine: load rules, make queries, etc."""

from datetime import datetime, timedelta
from pathlib import Path

from _polar_lib import lib

from .cache import Cache
from .errors import get_error
from .exceptions import PolarApiException, PolarRuntimeException
from .ffi import ffi_serialize, load_str, check_result, is_null, to_c_str, Predicate
from .query import Query, QueryResult


CLASSES = {}
CONSTRUCTORS = {}


class Polar:
    """Polar API"""

    def __init__(self, classes=CLASSES, constructors=CONSTRUCTORS):
        self.polar = lib.polar_new()
        self.cache = Cache(self.polar, classes=classes, constructors=constructors)
        self.load_queue = []
        self.constructors = constructors

        # Register built-in classes.
        self.register_class(datetime, name="Datetime")
        self.register_class(timedelta, name="Timedelta")

    def __del__(self):
        del self.cache
        lib.polar_free(self.polar)

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
        load_str(self.polar, string, None, self.run)

    def clear(self):
        """Clear all facts and internal Polar classes from the knowledge base."""
        self.load_queue = []
        lib.polar_free(self.polar)
        self.polar = None
        self.polar = lib.polar_new()

    def query(self, query, single=False):
        """Query for a predicate, parsing it if necessary.

        :param query: The predicate to query for.
        :param single: Whether to stop after the first result.

        :return: The result of the query.
        """
        self._load_queued_files()

        if isinstance(query, str):
            query = check_result(lib.polar_new_query(self.polar, to_c_str(query)))
        elif isinstance(query, Predicate):
            query = check_result(
                lib.polar_new_query_from_term(
                    self.polar, ffi_serialize(self.cache.to_polar_term(query))
                )
            )
        else:
            raise PolarApiException(f"Can not query for {query}")

        results = []
        for result in self.run(query):
            results.append(result)
            if single:
                break
        return QueryResult(results)

    def query_predicate(self, name, *args, **kwargs):
        """Query for predicate with name ``name`` and args ``args``.

        :param name: The name of the predicate to query.
        :param args: Arguments for the predicate.

        :return: The result of the query.
        """
        return self.query(Predicate(name=name, args=args), **kwargs)

    def run(self, query):
        """Send an FFI query object to a new Query object for evaluation."""
        return Query(self.polar, cache=self.cache).run(query)

    def repl(self):
        self._load_queued_files()
        while True:
            query = lib.polar_query_from_repl(self.polar)
            had_result = False
            if is_null(query):
                print("Query error: ", get_error())
                break
            for res in self.run(query):
                had_result = True
                print(f"Result: {res}")
            if not had_result:
                print("False")

    def register_class(self, cls, *, name=None, from_polar=None):
        """Register `cls` as a class accessible by Polar. `from_polar` can
        either be a method or a string. In the case of a string, Polar will
        look for the method using `getattr(cls, from_polar)`."""
        cls_name = cls.__name__ if name is None else name
        self.cache.cache_class(cls, cls_name, from_polar)
        self.register_constant(cls_name, cls)

    def register_constant(self, name, value):
        """Register `value` as a Polar constant variable called `name`."""
        name = to_c_str(name)
        value = ffi_serialize(self.cache.to_polar_term(value))
        lib.polar_register_constant(self.polar, name, value)

    def _load_queued_files(self):
        """Load queued policy files into the knowledge base."""
        while self.load_queue:
            filename = self.load_queue.pop(0)
            with open(filename) as file:
                load_str(self.polar, file.read(), filename, self.run)
