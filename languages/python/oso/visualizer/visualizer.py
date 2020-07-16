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


def build_tree(trace):
    if not trace:
        return None

    def sig(x):
        """ Generate a new name for a node """
        # @Note: This should just use term ids but we don't always have them.
        # Need to make sure we generate new term ids for all arguments to a predicate created from python.
        return str(abs(hash(str(x))))

    graph = {"nodes": set(), "edges": set()}

    def walk(trace, parent):
        root_node = parent is None
        node_kind = [*trace["node"]][0]
        if node_kind == "Term":
            term = trace["node"]["Term"]
            # @TODO: Put code context on traces directly.
            graph["nodes"].add((sig(trace), trace["polar_str"], "{}", root_node))
        elif node_kind == "Rule":
            rule = trace["node"]["Rule"]
            graph["nodes"].add((sig(trace), trace["polar_str"], "{}", root_node))
        else:
            print(f"Error: Unknown trace node kind {node_kind}")

        if parent:
            if sig(parent) == sig(trace):
                pass
            elif (sig(parent), sig(trace)) in graph["edges"]:
                pass
            else:
                graph["edges"].add((sig(parent), sig(trace)))

        for child in trace["children"]:
            walk(child, trace)

    walk(trace, None)

    return graph


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
        tree = build_tree(trace)

        if tree:
            tree["nodes"] = list(tree["nodes"])
            tree["edges"] = list(tree["edges"])
            return jsonify(tree)
        return trace
    return {}, 404
