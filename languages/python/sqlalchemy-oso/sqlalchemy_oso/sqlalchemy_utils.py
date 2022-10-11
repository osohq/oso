"""Utilities for interacting with SQLAlchemy types.

This module mostly deals with detecting which entities are involved in a query
that is about to be executed.

We must detect all entities properly to apply authorization.
"""
import sqlalchemy
from sqlalchemy import inspect
from sqlalchemy.orm.util import AliasedClass, AliasedInsp


def to_class(entity):
    """Get mapped class from SQLAlchemy entity."""
    if isinstance(entity, AliasedClass):
        return inspect(entity).class_
    elif inspect(entity, False) is not None:
        return inspect(entity).class_
    else:
        return entity


try:

    def all_entities_in_statement(statement):
        """
        Get all ORM entities that will be loaded in a select statement.

        The includes entities that will be loaded eagerly through relationships either specified in
        the query options or as default loader strategies on the model definition.

        https://docs.sqlalchemy.org/en/14/orm/loading_relationships.html#relationship-loading-with-loader-options
        """
        entities = get_column_entities(statement)
        entities |= get_joinedload_entities(statement)
        entities |= default_load_entities(entities)

        return set(map(to_class, entities))

    def get_column_entities(statement):
        """Get entities in statement that are referenced as columns.

        Examples::

            >> get_column_entities(query(A)) == {A}
            >> get_column_entities(query(A.field)) == {A}
            >> get_column_entities(query(A, B)) == {A, B})

        Does not include eager loaded entities.
        """

        def _entities_in_statement(statement):
            try:
                entities = (cd["entity"] for cd in statement.column_descriptions)
                return set(e for e in entities if e is not None)
            except AttributeError:
                return set()

        entities = _entities_in_statement(statement)

        # TODO(gj): currently walking way more than we have to. Probably
        # some points in the tree where we can safely call it good for that
        # branch and continue on to more fruitful pastures.
        for child in statement.get_children():
            entities |= get_column_entities(child)

        return entities

    def default_load_entities(entities, seen_relationships=None):
        """Find related entities that will be loaded on all queries to ``entities``
           due to the default loader strategy.

        For example::

            class A(Base):
                bs = relationship(B, lazy="joined")

        The relationship ``bs`` would be loaded eagerly whenever ``A`` is queried because
        `lazy="joined"`.

        :param entities: The entities to lookup default load entities for.
        """
        default_entities = set()

        for entity in entities:
            mapper = sqlalchemy.inspect(entity)
            # If the entity is an alias, get the mapper for the underlying entity.
            if isinstance(mapper, AliasedInsp):
                mapper = mapper.mapper

            relationships = mapper.relationships
            if seen_relationships is None:
                seen_relationships = set()
            for rel in relationships.values():
                if rel in seen_relationships:
                    # prevent infinitely recursing when we've already seen the relationship
                    continue
                seen_relationships.add(rel)
                # We only detect `"joined"` here because `"selectin"` and `"subquery"`
                # issue multiple queries that we capture in the `do_orm_execute` event
                # handler.
                if rel.lazy == "joined":
                    default_entities |= default_load_entities(
                        [rel.mapper], seen_relationships
                    )
                    default_entities.add(rel.mapper)

        return default_entities

    # Start POC code from @zzzeek (Mike Bayer)
    # TODO: Still needs to be generalized & support other options.

    # the structure we're dealing with is essentially:

    # (path, strategy, options)
    # where "path" indicates what it is we are loading,
    # like (A, A.bs, B, B.cs, C)
    # "strategy" is a tuple that keys to one of the loader strategies,
    # some of them apply to relationships and others to column attributes
    # then "options" is extra stuff like "innerjoin=True"
    def get_joinedload_entities(stmt):
        """Get extra entities that are loaded from a ``stmt`` due to joinedload
        options specified in the statement options.

        These entities will not be returned directly by the query, but will prepopulate
        relationships in the returned data.

        For example::

            get_joinedload_entities(query(A).options(joinedload(A.bs))) == {A, B}
        """
        # there are two kinds of options that both represent the same information,
        # just in different ways.  This is largely a product of legacy options
        # that have things like strings, i.e. joinedload("addresses").  note we
        # aren't covering that here, which is legacy form.  you can if you want
        # raise an exception if you detect that form here.

        entities = set()

        for opt in stmt._with_options:
            if hasattr(opt, "_to_bind"):
                # these options are called _UnboundLoad
                for b in opt._to_bind:
                    if ("lazy", "joined") in b.strategy:
                        # the "path" is a tuple showing the entity/relationships
                        # being targeted

                        # TODO check for wild card.
                        # TODO: Check whether entity is a string.
                        entities.add(b.path[-1].entity)
            elif hasattr(opt, "context"):
                # these options are called Load
                for key, loadopt in opt.context.items():
                    if key[0] == "loader" and ("lazy", "joined") in loadopt.strategy:
                        # the "path" is a tuple showing the entity/relationships
                        # being targeted

                        # TODO: Check for of_type.
                        # TODO: Check whether entity is a string, unsupported.
                        # TODO check for wild card.
                        entities.add(key[1][-1].entity)

        return entities

except ImportError:
    # This code should not be called for SQLAlchemy 1.4.
    def all_entities_in_statement(statement):
        raise NotImplementedError("Unsupported on SQLAlchemy < 1.4")
