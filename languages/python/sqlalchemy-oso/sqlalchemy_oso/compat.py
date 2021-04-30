"""SQLAlchemy version compatibility tools.

Keep us compatible with 1.3 and 1.4 by implementing wrappers when needed here.
"""


def iterate_model_classes(declarative_base):
    """Return an iterator of model classes that descend from this declarative base."""
    try:
        for name, model in declarative_base._decl_class_registry.items():
            if name == "_sa_module_registry":
                continue

            yield model
    except AttributeError:
        yield declarative_base  ## TODO ensure that 1.3 includes base in above iter
        for mapper in declarative_base.registry.mappers:
            yield mapper.class_
