"""Communicate with the Polar virtual machine: load rules, make queries, etc."""

from datetime import datetime, timedelta
from pathlib import Path
from pprint import pprint
import sys
import hashlib

from _polar_lib import lib

from .exceptions import (
    PolarApiException,
    PolarRuntimeException,
    ParserException,
    PolarFileAlreadyLoadedError,
    PolarFileContentsChangedError,
    PolarFileNameChangedError,
)
from .ffi import Polar as FfiPolar, Query as FfiQuery
from .host import Host
from .query import Query, QueryResult
from .predicate import Predicate
from .variable import Variable


CLASSES = {}
CONSTRUCTORS = {}


class Polar:
    """Polar API"""

    def __init__(self, classes=CLASSES, constructors=CONSTRUCTORS):
        self.ffi_polar = FfiPolar()
        self.host = Host(self.ffi_polar)
        self.loaded_names = {}
        self.loaded_contents = {}

        # Register built-in classes.
        self.register_class(bool, name="Boolean")
        self.register_class(int, name="Integer")
        self.register_class(float, name="Float")
        self.register_class(list, name="List")
        self.register_class(dict, name="Dictionary")
        self.register_class(str, name="String")
        self.register_class(datetime, name="Datetime")
        self.register_class(timedelta, name="Timedelta")

        # Pre-registered classes.
        for name, cls in classes.items():
            self.register_class(cls, name=name, from_polar=constructors[name])

    def __del__(self):
        del self.host
        del self.ffi_polar

    def clear(self):
        self.loaded_names = {}
        self.loaded_contents = {}
        del self.ffi_polar
        self.ffi_polar = FfiPolar()

    def load_file(self, policy_file):
        """Load in polar policies. By default, defers loading of knowledge base
        until a query is made."""
        policy_file = Path(policy_file)
        extension = policy_file.suffix
        if not extension == ".polar":
            raise PolarApiException(
                f"Polar files must have .polar extension. Offending file: {policy_file}"
            )

        fname = str(policy_file)

        # Checksum file contents
        try:
            with open(fname, "rb") as f:
                file_data = f.read()
            fhash = hashlib.md5(file_data).hexdigest()
        except FileNotFoundError:
            raise PolarApiException(f"Could not find file: {policy_file}")

        if fname in self.loaded_names.keys():
            if self.loaded_names.get(fname) == fhash:
                raise PolarFileAlreadyLoadedError(
                    f"File {fname} has already been loaded."
                )
            else:
                raise PolarFileContentsChangedError(
                    f"A file with the name {fname}, but different contents, has already been loaded."
                )
        elif fhash in self.loaded_contents.keys():
            raise PolarFileNameChangedError(
                f"A file with the same contents as {fname} named {self.loaded_contents.get(fhash)} has already been loaded."
            )
        else:
            self.load_str(file_data.decode("utf-8"), policy_file)
            self.loaded_names[fname] = fhash
            self.loaded_contents[fhash] = fname

    def load_str(self, string, filename=None):
        """Load a Polar string, checking that all inline queries succeed."""
        self.ffi_polar.load_str(string, filename)

        # check inline queries
        while True:
            query = self.ffi_polar.next_inline_query()
            if query is None:  # Load is done
                break
            else:
                try:
                    next(Query(query, host=self.host.copy()).run())
                except StopIteration:
                    raise PolarRuntimeException("Inline query in file failed.")

    def query(self, query):
        """Query for a predicate, parsing it if necessary.

        :param query: The predicate to query for.

        :return: The result of the query.
        """
        host = self.host.copy()
        if isinstance(query, str):
            query = self.ffi_polar.new_query_from_str(query)
        elif isinstance(query, Predicate):
            query = self.ffi_polar.new_query_from_term(host.to_polar_term(query))
        else:
            raise PolarApiException(f"Can not query for {query}")

        for res in Query(query, host=host).run():
            yield res

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

        while True:
            try:
                query = input("query> ").strip(";")
            except EOFError:
                return
            try:
                ffi_query = self.ffi_polar.new_query_from_str(query)
            except ParserException as e:
                print("Parse error: ", str(e.value()))
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
        self.ffi_polar.register_constant(name, self.host.to_polar_term(value))


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
