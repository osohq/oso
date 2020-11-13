from sqlalchemy.orm.session import Session
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import RelationshipProperty, ColumnProperty
from sqlalchemy.sql.expression import ClauseElement, BinaryExpression, and_

from polar.partial import Partial
from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError

def partial_to_query(expression: Expression, session: Session, model) -> Query:
    """Convert constraints in ``partial`` to a query over ``model``."""
    # Top level operation must be and.
    query = session.query(model)

    print(expression)

    expr = translate_expr(expression, session, model)
    if expr is not None:
        return query.filter(expr)

    return query

# Returns None or the translated expression.
def translate_expr(expression: Expression, session: Session, model):
    assert isinstance(expression, Expression)
    if expression.operator == 'Eq' or expression.operator == 'Unify':
        return compare_expr(expression, session, model)
    elif expression.operator == 'Isa':
        assert expression.args[1].tag == model.__name__
        return None
    elif expression.operator == 'In':
        return translate_in(expression, session, model)
    elif expression.operator == 'And':
        return translate_and_expr(expression, session, model)
    else:
        raise UnsupportedError(f"Unsupported {expression}")

def translate_and_expr(expression: Expression, session: Session, model) -> BinaryExpression:
    expr = and_()
    assert expression.operator == "And"
    for expression in expression.args:
        translated = translate_expr(expression, session, model)
        if translated is None:
            continue

        expr = expr & translated

    return expr

def compare_expr(expression: Expression, session: Session, model) -> BinaryExpression:
    left = expression.args[0]
    right = expression.args[1]

    if dot_op_path(left):
        path = dot_op_path(left)
        value = right
    else:
        path = dot_op_path(right)
        assert path
        value = left

    return translate_comparison(path, value, model)



def translate_sub_expression(path, sub_expression, session: Session, model):
    """Translate a sub_expression path on model."""
    if len(path) == 1:
        # Complicated... there are multiple base cases depending on the type of
        # sub_expression... see if we can improve this. Er I guess these should be diff functions.
        if sub_expression.op in ['in'

    if len(path) == 0:
        return translate_expr(sub_expression, session, model)
    else:
        property = getattr(model, path[0])
        assert isinstance(property.property, RelationshipProperty)
        relationship = property.property
        model = property.entity.class_

        if not relationship.uselist:
            return property.has(
                translate_sub_expression(path[1:], sub_expression, session, model))
        else:
            return property.any(translate_sub_expression(path[1:], sub_expression, session, model))



def translate_comparison(path, value, model):
    """Translate a comparison operation of ``path`` = ``value`` on ``model``."""
    if len(path) == 1:
        property = getattr(model, path[0])
        return property == value
    else:
        # TODO this has assumes that nested relationships are always
        # a scalar attribute... it also probably isn't as efficient as a
        # join usually, so we may want to translate differently.
        property = getattr(model, path[0])
        assert isinstance(property.property, RelationshipProperty)
        relationship = property.property

        if not relationship.uselist:
            return property.has(
                translate_comparison(path[1:], value, property.entity.class_))
        else:
            # TODO (dhatch): Should this assert? This would come from comparing
            # something against a multi-valued property
            return property.any(
                translate_comparison(path[1:], value, property.entity.class_))


# TODO (dhatch): Move this helper into base.
def dot_op_path(expr):
    """Get the path components of a lookup.

    The path is returned as a list.

    _this.created_by => ['created_by']
    _this.created_by.username => ['created_by', 'username']

    None is returned if input is not a dot operation.
    """
    if not isinstance(expr, Expression):
        return None

    if not expr.operator == "Dot":
        return None

    assert len(expr.args) == 2

    if expr.args[0] == Variable('_this'):
        return [expr.args[1]]

    return dot_op_path(expr.args[0]) + [expr.args[1]]

def translate_in(expression, session, model):
    assert expression.operator == 'In'
    left = expression.args[0]
    right = expression.args[1]

    # IN means at least something must be contained in the property.

    # There are two possible types of in operations. In both, the right hand side
    # should be a dot op.

    # Partial In: LHS is an expression
    if isinstance(left, Expression):
        path = dot_op_path(right)
        assert path

        # (_this.is_public = true) in (_this.tags)
        # [tags], model = Post
        return translate_in_path(path, left, session, model)
    else:
        # TODO (dhatch) Missing check, left type must match type of the target?
        path = dot_op_path(right)
        assert path
        return translate_contains(path, left, session, model)

    # Contains: LHS is not an expression.

# TODO (dhatch): Test multiple levels w/ this function.

def translate_in_path(path, sub_expression, session, model):
    # TODO use this for dot op path, but have the sub expression be the lookup / comparison.
    """Translate an in op like (EXPR) in PATH"""
    if len(path) == 0:
        # _this.is_public = true
        # model: Tag
        return translate_expr(sub_expression, session, model)
    else:
        property = getattr(model, path[0])
        assert isinstance(property.property, RelationshipProperty)
        relationship = property.property
        model = property.entity.class_

        # model = Tag

        if not relationship.uselist:
            return property.has(
                translate_in_path(path[1:], sub_expression, session, model))
        else:
            # post.tags.any()
            return property.any(translate_in_path(path[1:], sub_expression, session, model))

def translate_contains(path, value, session, model):
    # TODO use this for dot op path, but have the sub expression be the lookup / comparison.
    """Translate an in op like (EXPR) in PATH"""
    if len(path) == 1:
        # _this.is_public = true
        # model: Tag
        property = getattr(model, path[0])
        model = property.entity.class_
        assert isinstance(value, model)
        return property.contains(value)
    else:
        # TODO I DON"T THINK THIS IS RIGHT
        property = getattr(model, path[0])
        assert isinstance(property.property, RelationshipProperty)
        relationship = property.property
        model = property.entity.class_

        # model = Tag

        if not relationship.uselist:
            return property.has(
                translate_contains(path[1:], sub_expression, session, model))
        else:
            # post.tags.any()
            return property.any(translate_contains(path[1:], sub_expression, session, model))
