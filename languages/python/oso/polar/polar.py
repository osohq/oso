"""Communicate with the Polar virtual machine: load rules, make queries, etc."""

from datetime import datetime, timedelta
import inspect
import os
from pathlib import Path
import sys
from typing import Dict

try:
    # importing readline on compatible platforms
    # changes how `input` works for the REPL
    import readline  # noqa: F401
except ImportError:
    pass

from .exceptions import (
    PolarRuntimeError,
    InlineQueryFailedError,
    ParserError,
    PolarFileExtensionError,
    PolarFileNotFoundError,
    InvalidQueryTypeError,
)
from .ffi import Polar as FfiPolar
from .host import Host
from .query import Query
from .predicate import Predicate
from .variable import Variable
from .expression import Expression, Pattern
from .data_filtering import serialize_types, filter_data


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


CLASSES: Dict[str, type] = {}


class Polar:
    """Polar API"""

    def __init__(self, classes=CLASSES):
        self.ffi_polar = FfiPolar()
        self.host = Host(self.ffi_polar)
        self.ffi_polar.set_message_enricher(self.host.enrich_message)
        # TODO(gj): rename to _oso_roles_enabled
        self._polar_roles_enabled = False

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

        class InternalRolesHelpers:
            @staticmethod
            def join(separator, left, right):
                return separator.join([left, right])

        self.register_class(InternalRolesHelpers, name="__oso_internal_roles_helpers__")

        # Pre-registered classes.
        for name, cls in classes.items():
            self.register_class(cls, name=name)

    def __del__(self):
        del self.host
        del self.ffi_polar

    def enable_roles(self):
        if not self._polar_roles_enabled:

            self.ffi_polar.enable_roles()
            self._polar_roles_enabled = True

            # validate config
            validation_query_results = []
            while True:
                query = self.ffi_polar.next_inline_query()
                if query is None:  # Load is done
                    break
                try:
                    host = self.host.copy()
                    host.set_accept_expression(True)
                    validation_query_results.append(list(Query(query, host=host).run()))
                except StopIteration:
                    source = query.source()
                    raise InlineQueryFailedError(source.get())

            # turn bindings back into polar
            for results in validation_query_results:
                for result in results:
                    for k, v in result["bindings"].items():
                        result["bindings"][k] = host.to_polar(v)

            self.ffi_polar.validate_roles_config(validation_query_results)

    def load_file(self, policy_file):
        """Load Polar policy from a ".polar" file."""
        policy_file = Path(policy_file)
        extension = policy_file.suffix
        fname = str(policy_file)
        if not extension == ".polar":
            raise PolarFileExtensionError(fname)

        try:
            with open(fname, "rb") as f:
                file_data = f.read()
        except FileNotFoundError:
            raise PolarFileNotFoundError(fname)

        self.load_str(file_data.decode("utf-8"), policy_file)

    def load_str(self, string, filename=None):
        """Load a Polar string, checking that all inline queries succeed."""
        # Get MRO of all registered classes
        # NOTE: not ideal that the MRO gets updated each time load_str is
        # called, but since we are planning to move to only calling load once
        # with the include feature, I think it's okay for now.
        for rec in self.host.distinct_user_types():
            mro = [
                self.host.types[c].id
                for c in inspect.getmro(rec.cls)
                if c in self.host.types
            ]
            self.ffi_polar.register_mro(rec.name, mro)

        self.ffi_polar.load(string, filename)

        # check inline queries
        while True:
            query = self.ffi_polar.next_inline_query()
            if query is None:  # Load is done
                break
            else:
                try:
                    next(Query(query, host=self.host.copy()).run())
                except StopIteration:
                    source = query.source()
                    raise InlineQueryFailedError(source.get())

        # If roles are enabled, re-validate config when new rules are loaded.
        if self._polar_roles_enabled:
            self._polar_roles_enabled = False
            self.enable_roles()

    def clear_rules(self):
        self.ffi_polar.clear_rules()
        self._polar_roles_enabled = False

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

        for res in Query(query, host=host, bindings=bindings).run():
            yield res

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
        for f in files:
            self.load_file(f)

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

    def register_class(self, cls, *, name=None, types=None, fetcher=lambda _: []):
        """Register `cls` as a class accessible by Polar."""
        cls_name = self.host.cache_class(cls, name=name, fields=types, fetcher=fetcher)
        self.register_constant(cls, cls_name)

    def register_constant(self, value, name):
        """Register `value` as a Polar constant variable called `name`."""
        self.ffi_polar.register_constant(self.host.to_polar(value), name)

    def get_class(self, name):
        """Return class registered for ``name``.

        :raises UnregisteredClassError: If the class is not registered.
        """
        return self.host.get_class(name)

    def get_allowed_resources(self, actor, action, cls) -> list:
        """
        Returns all the resources the actor is allowed to perform action on.

        :param actor: The actor for whom to collect allowed resources.

        :param action: The action that user wants to perform.

        :param cls: The type of the resources.

        :return: A list of the unique allowed resources.
        """
        # Data filtering.
        resource = Variable("resource")
        # Get registered class name somehow
        class_name = self.host.types[cls].name
        constraint = Expression(
            "And", [Expression("Isa", [resource, Pattern(class_name, {})])]
        )
        results = list(
            self.query_rule(
                "allow",
                actor,
                action,
                resource,
                bindings={"resource": constraint},
                accept_expression=True,
            )
        )

        complete, partial = [], []

        for result in results:
            for k, v in result["bindings"].items():
                if isinstance(v, Expression):
                    partial.append({"bindings": {k: self.host.to_polar(v)}})
                else:
                    complete.append(v)

        types = serialize_types(self.host.distinct_user_types(), self.host.types)
        plan = self.ffi_polar.build_filter_plan(types, partial, "resource", class_name)
        complete += filter_data(self, plan)
        return complete


def polar_class(_cls=None, *, name=None):
    """Decorator to register a Python class with Polar.
    An alternative to ``register_class()``."""

    def wrap(cls):
        cls_name = cls.__name__ if name is None else name
        CLASSES[cls_name] = cls
        return cls

    if _cls is None:
        return wrap

    return wrap(_cls)
