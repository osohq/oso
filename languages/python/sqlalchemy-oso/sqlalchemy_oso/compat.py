"""SQLAlchemy version compatibility tools.

Keep us compatible with multiple SQLAlchemy versions by implementing wrappers
when needed here.
"""
import sqlalchemy
from packaging.version import parse

version = parse(sqlalchemy.__version__)  # type: ignore
USING_SQLAlchemy_v1_3 = version >= parse("1.3") and version < parse("1.4")


def iterate_model_classes(base_or_registry):
    """Return an iterator of model classes that descend from a declarative base
    (SQLAlchemy 1.3 or 1.4) or exist in a registry (SQLAlchemy 1.4)."""
    try:  # 1.3 declarative base.
        # TODO (dhatch): Not sure this is legit b/c it uses an internal interface?
        models = base_or_registry._decl_class_registry.items()
        for name, model in models:
            if name != "_sa_module_registry":
                if isinstance(
                    model, sqlalchemy.ext.declarative.clsregistry._MultipleClassMarker
                ):
                    for model_ref in model.contents:
                        yield model_ref()
                else:
                    yield model
    except AttributeError:
        try:  # 1.4 declarative base.
            mappers = base_or_registry.registry.mappers
        except AttributeError:  # 1.4 registry.
            mappers = base_or_registry.mappers
        yield from {mapper.class_ for mapper in mappers}
