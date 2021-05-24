"""SQLAlchemy version compatibility tools.

Keep us compatible with 1.3 and 1.4 by implementing wrappers when needed here.
"""
from packaging import version

import sqlalchemy

AT_LEAST_SQLALCHEMY_VERSION_1_4 = version.parse(
    sqlalchemy.__version__  # type: ignore
) >= version.parse("1.4")

if AT_LEAST_SQLALCHEMY_VERSION_1_4:

    def iterate_model_classes(base_or_registry):
        """Return an iterator of model classes that descend from a declarative
        base or exist in a registry (SQLAlchemy 1.4)."""
        try:
            mappers = base_or_registry.mappers  # Registry.
        except AttributeError:
            mappers = base_or_registry.registry.mappers  # Base.
        yield from {mapper.class_ for mapper in mappers}


else:

    def iterate_model_classes(base_or_registry):
        """Return an iterator of model classes that descend from a declarative
        base (SQLAlchemy 1.3)."""

        # TODO (dhatch): Not sure this is legit b/c it uses an internal interface?
        models = base_or_registry._decl_class_registry.items()
        yield from {model for name, model in models if name != "_sa_module_registry"}
