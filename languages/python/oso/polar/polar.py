"""Communicate with the Polar virtual machine: load rules, make queries, etc."""

from datetime import datetime, timedelta
import os
from pathlib import Path
import sys
from typing import List, Union, Optional

from .exceptions import (
    PolarRuntimeError,
    InlineQueryFailedError,
    ParserError,
    PolarFileExtensionError,
    PolarFileNotFoundError,
    InvalidQueryTypeError,
)
from .ffi import Polar as FfiPolar, PolarSource as Source
from .host import Host
from .query import Query
from .predicate import Predicate
from .variable import Variable
from .expression import Expression, Pattern
from .data_filtering import serialize_types
from .data import DataFilter


class Polar:
    """Polar API"""

    def __init__(self) -> None:
        self.ffi_polar = FfiPolar()
        self.host = Host(self.ffi_polar)
        self.ffi_polar.set_message_enricher(self.host.enrich_message)

        # Register global constants.
        self.register_constant(None, name="nil")

        # Register built-in classes.
        self.register_class(bool, name="Boolean")
        self.register_class(int, name="Integer")
        self.register_class(float, name="Float")
        self.register_class(list, name="List")
        self.register_class(dict, name="Dictionary")
        self.register_class(str, name="String")
        self.register_class(datetime, name="Datetime")
        self.register_class(timedelta, name="Timedelta")

    def __del__(self) -> None:
        del self.host
        del self.ffi_polar

    def load_files(self, filenames: Optional[List[Union[Path, str]]] = None) -> None:
        """Load Polar policy from ".polar" files."""
        if filenames is None:
            filenames = []

        if not filenames:
            return

        sources: List[Source] = []

        for filename in filenames:
            path = Path(filename)
            extension = path.suffix
            filename = str(path)
            if extension != ".polar":
                raise PolarFileExtensionError(filename)

            try:
                with open(filename, "rb") as f:
                    src = f.read().decode("utf-8")
                    sources.append(Source(src, filename))
            except FileNotFoundError:
                raise PolarFileNotFoundError(filename)

        self._load_sources(sources)

    def load_file(self, filename: Union[Path, str]) -> None:
        """Load Polar policy from a ".polar" file.

        `Oso.load_file` has been deprecated in favor of `Oso.load_files` as of
        the 0.20 release. Please see changelog for migration instructions:
        https://docs.osohq.com/project/changelogs/2021-09-15.html
        """
        print(
            "`Oso.load_file` has been deprecated in favor of `Oso.load_files` as of the 0.20 release.\n\n"
            + "Please see changelog for migration instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html",
            file=sys.stderr,
        )
        self.load_files([filename])

    def load_str(self, string: str) -> None:
        """Load a Polar string, checking that all inline queries succeed."""
        # NOTE: not ideal that the MRO gets updated each time load_str is
        # called, but since we are planning to move to only calling load once
        # with the include feature, I think it's okay for now.
        self._load_sources([Source(string)])

    # Register MROs, load Polar code, and check inline queries.
    def _load_sources(self, sources: List[Source]) -> None:
        self.ffi_polar.load(sources)
        self.check_inline_queries()

    def check_inline_queries(self) -> None:
        while True:
            query = self.ffi_polar.next_inline_query()
            if query is None:  # Load is done
                break
            else:
                try:
                    next(Query(query, host=self.host.copy()).run())
                except StopIteration:
                    source = query.source()
                    raise InlineQueryFailedError(source)

    def clear_rules(self) -> None:
        self.ffi_polar.clear_rules()

    def query(self, query, *, bindings=None, accept_expression=False):
        """Query for a predicate, parsing it if necessary.

        :param query: The predicate to query for.

        :return: The result of the query.
        """
        host = self.host.copy()
        host.set_accept_expression(accept_expression)

        if isinstance(query, str):
            query = self.ffi_polar.new_query_from_str(query)
        elif isinstance(query, Predicate):
            query = self.ffi_polar.new_query_from_term(host.to_polar(query))
        else:
            raise InvalidQueryTypeError()

        yield from Query(query, host=host, bindings=bindings).run()

    def query_rule(self, name, *args, **kwargs):
        """Query for rule with name ``name`` and arguments ``args``.

        :param name: The name of the predicate to query.
        :param args: Arguments for the predicate.

        :return: The result of the query.
        """
        return self.query(Predicate(name=name, args=args), **kwargs)

    def query_rule_once(self, name, *args, **kwargs):
        """Check a rule with name ``name`` and arguments ``args``.

        :param name: The name of the predicate to query.
        :param args: Arguments for the predicate.

        :return: True if the query has any results, False otherwise.
        """
        try:
            next(self.query(Predicate(name=name, args=args), **kwargs))
            return True
        except StopIteration:
            return False

    def repl(self, files=[]):
        """Start an interactive REPL session."""
        try:
            # importing readline on compatible platforms
            # changes how `input` works for the REPL
            import readline  # noqa: F401
        except ImportError:
            pass

        # https://github.com/django/django/blob/3e753d3de33469493b1f0947a2e0152c4000ed40/django/core/management/color.py
        def supports_color():
            supported_platform = sys.platform != "win32" or "ANSICON" in os.environ
            is_a_tty = hasattr(sys.stdout, "isatty") and sys.stdout.isatty()
            return supported_platform and is_a_tty

        RESET = ""
        FG_BLUE = ""
        FG_RED = ""

        if supports_color():
            # \001 and \002 signal these should be ignored by readline. Explanation of
            # the issue: https://stackoverflow.com/a/9468954/390293. Issue has been
            # observed in the Python REPL on Linux by @samscott89 and @plotnick, but
            # not on macOS or Windows (with readline installed) or in the Ruby or
            # Node.js REPLs, both of which also use readline.
            RESET = "\001\x1b[0m\002"
            FG_BLUE = "\001\x1b[34m\002"
            FG_RED = "\001\x1b[31m\002"

        def print_error(error):
            print(FG_RED + type(error).__name__ + RESET)
            print(error)

        self.load_files(files)

        while True:
            try:
                query = input(FG_BLUE + "query> " + RESET).strip(";")
            except (EOFError, KeyboardInterrupt):
                return
            try:
                ffi_query = self.ffi_polar.new_query_from_str(query)
            except ParserError as e:
                print_error(e)
                continue

            host = self.host.copy()
            host.set_accept_expression(True)
            result = False
            try:
                query = Query(ffi_query, host=host).run()
                for res in query:
                    result = True
                    bindings = res["bindings"]
                    if bindings:
                        for variable, value in bindings.items():
                            print(variable + " = " + repr(value))
                    else:
                        print(True)
            except PolarRuntimeError as e:
                print_error(e)
                continue
            if not result:
                print(False)

    def register_class(
        self,
        cls,
        *,
        name=None,
        fields=None,
    ):
        """
        Register `cls` as a class accessible by Polar.

        :param name:
            Optionally specify the name for the class inside of Polar. Defaults
            to `cls.__name__`
        :param fields:
            Optional dict mapping field names to types or Relation objects for
            data filtering.
        """
        # TODO: let's add example usage here or at least a proper docstring for the arguments

        name = self.host.cache_class(
            cls,
            name=name,
            fields=fields,
        )
        self.register_constant(cls, name)
        self.host.register_mros()

    def register_constant(self, value, name):
        """
        Register `value` as a Polar constant variable called `name`.

        :param value:
            The value to register as a constant.
        :param name:
            The name under which the constant will be visible in Polar.
        """
        self.ffi_polar.register_constant(self.host.to_polar(value), name)

    def get_class(self, name):
        """Return class registered for ``name``.

        :raises UnregisteredClassError: If the class is not registered.
        """
        return self.host.get_class(name)

    def partial_query(self, actor, action, resource_cls):
        resource = Variable("resource")
        class_name = self.host.types[resource_cls].name
        constraint = Expression(
            "And", [Expression("Isa", [resource, Pattern(class_name, {})])]
        )

        query = self.query_rule(
            "allow",
            actor,
            action,
            resource,
            bindings={"resource": constraint},
            accept_expression=True,
        )

        return [
            {"bindings": {k: self.host.to_polar(v)}}
            for result in query
            for k, v in result["bindings"].items()
        ]

    def is_new_data_filtering_configured(self) -> bool:
        return self.host.adapter is not None

    def new_authorized_query(self, actor, action, resource_cls):
        results = self.partial_query(actor, action, resource_cls)

        types = serialize_types(self.host.distinct_user_types(), self.host.types)
        class_name = self.host.types[resource_cls].name
        plan = self.ffi_polar.build_data_filter(types, results, "resource", class_name)

        return self.host.adapter.build_query(DataFilter.parse(self, plan))
