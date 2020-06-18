# Depends on
# bootstrap-flask, flask, and graphviz
# pip install flask bootstrap-flask graphviz && python -m flask run

# External imports
from collections import deque
from datetime import datetime
import sys

try:
    from flask import (
        Blueprint,
        current_app,
        jsonify,
        redirect,
        render_template,
        Markup,
        url_for,
    )
    from flask_bootstrap import Bootstrap
    from graphviz import Digraph
except ImportError:
    print(
        "Missing optional dependencies for visualizer. Install oso[visualizer].",
        file=sys.stderr,
    )
    raise

import json

from oso.audit import AuditLog

# from polar.parser import Fact, Rule, Variable
# from polar.facts import Facts
# from polar.classes import Class, Instance

oso_viz = Blueprint(
    "oso_visualizer", __name__, static_folder="static", template_folder="templates"
)


def load_viz(app, policy):
    bootstrap = Bootstrap(app)
    app.register_blueprint(oso_viz, url_prefix="/viz")
    app.config["policy"] = policy


OSO_BLUE = "#005b96"
OSO_BLUE_DARKEST = "#011f4b"
OSO_BLUE_LIGHTEST = "#b3cde0"


def db():
    db = current_app.config["policy"]
    return db


###############################################################################
################################### HELPERS ###################################
###############################################################################


# def contains_var(x):
#     if isinstance(x, Variable):
#         return True
#     elif isinstance(x, (list, tuple)):
#         return any(contains_var(y) for y in x)
#     else:
#         return False


# assert contains_var(Variable("x"))
# assert not contains_var("x")
# assert contains_var(("x", Variable("x")))
# assert contains_var(("x", ("x", Variable("x"))))
# assert not contains_var(("x", ("x", ("x"))))


###############################################################################
#################################### AUDIT ####################################
###############################################################################


# def build_tree(trace):
#     if not trace:
#         return None
#     assert isinstance(trace[1], Rule)

#     def sig(x):
#         """Generate a name for a node."""
#         return str(abs(hash(str(x))))

#     graph = {
#         "nodes": set(),
#         "edges": set(),
#     }

#     def node(term, label):
#         is_root_node = isinstance(term, Facts)
#         code_context = getattr(term, "code_context", None)
#         if code_context:
#             code_context = {
#                 "lineno": term.code_context.lineno,
#                 "column": term.code_context.column,
#                 "filename": str(term.code_context.filename),
#             }
#         graph["nodes"].add((sig(term), label, json.dumps(code_context), is_root_node))

#     def edge(x, y):
#         if sig(x) == sig(y):
#             pass
#         elif (sig(y), sig(x)) in graph["edges"]:
#             pass
#         else:
#             graph["edges"].add((sig(x), sig(y)))

#     def walk_tree(tree, parent):
#         if isinstance(tree, Fact):
#             # Make a node, an edge to the parent, and walk any args.
#             (head, *args) = tree
#             node(tree, str(tree))
#             edge(tree, parent)
#             if not contains_var(args):
#                 walk_tree(args, tree)
#         elif isinstance(tree, (tuple, list)):
#             # Recursively walk.
#             for x in tree:
#                 walk_tree(x, parent)

#     # Make a root (KB) node.
#     kb = trace[0]
#     assert isinstance(kb, Facts)
#     node(kb, sig(kb))

#     # Make nodes for each other element of the trace.
#     # There should be a trace element for each conjunct
#     # on the RHS of a rule. We need to track those manually,
#     # because the trace doesn't have parent pointers.
#     stack = deque([(kb, [])])  # parent, RHS

#     def top():
#         return stack[-1]

#     def par():
#         return top()[0]

#     def rhs():
#         return top()[1]

#     def push(parent, rhs):
#         stack.append([parent, deque(rhs)])

#     def pop():
#         try:
#             return rhs().popleft()
#         except IndexError:
#             return stack.pop()

#     # Walk the trace.
#     for step in trace[1:]:
#         if len(step) > 1:
#             # A rule.
#             parent = par()
#             push(step[0], step[1:])
#         else:
#             # A fact.
#             assert len(step) == 1
#             pop()
#             parent = par()
#         walk_tree(step[0], parent)

#     return graph


# def to_dot(graph):
#     if not graph:
#         return None

#     dot = Digraph(
#         format="svg",
#         graph_attr={"rankdir": "BT"},
#         node_attr={"color": OSO_BLUE_LIGHTEST, "fontname": "arial", "style": "bold"},
#     )

#     for node in graph["nodes"]:
#         attributes = {}
#         if node[2]:
#             attributes = {"href": node[2], "fontcolor": OSO_BLUE}

#         dot.node(node[0], node[1], _attributes=attributes)

#     for x, y in graph["edges"]:
#         dot.edge(x, y)

#     svg = dot.pipe().decode("utf-8")
#     return svg


@oso_viz.route("/events")
def event_index():
    events = [
        (
            str(event.id),
            event.timestamp,
            str(event.actor),
            str(event.action),
            str(event.resource),
            event.success,
        )
        for event in AuditLog().iter()
    ]
    return render_template(
        "audit_table.html", heading="Events", events=events, endpoint=".event_show",
    )


@oso_viz.route("/events/clear")
def event_clear():
    AuditLog().clear()
    return redirect(url_for(".event_index"))


@oso_viz.route("/events/<int:id>")
def event_show(id):
    event = AuditLog().get(id)
    if not event:
        return redirect(url_for(".event_index"))

    actor = str(event.actor)
    action = str(event.action)
    resource = str(event.resource)
    success = event.success

    return render_template(
        "event.html",
        heading="Event",
        actor=actor,
        action=action,
        resource=resource,
        result=success,
    )
    # TODO (leina): return render template for event as item


@oso_viz.route("/event_json/<int:id>", methods=["GET"])
def event_json(id):
    event = AuditLog().get(id)
    trace = event.trace

    if trace:
        # tree = build_tree(trace)

        # if tree:
        #     tree["nodes"] = list(tree["nodes"])
        #     tree["edges"] = list(tree["edges"])
        #     return jsonify(tree)
        return trace
    return {}, 404
