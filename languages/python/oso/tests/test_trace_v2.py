import json
import tempfile

import pytest
from graphviz import Digraph

from oso import Oso, Variable, Predicate


def build_trace_file():
    oso = Oso()
    oso.load_str("f(1); f(2);")

    with tempfile.NamedTemporaryFile(suffix=".polar") as f:
        f.write(
            """
            f(x, y) if x > 0 and y < 1 and x < 5;
            f(x, _) if x = 1;
            f(x, y) if x = 3 and y = 4;
            f(x, y) if x = 1 and y = 4;
            f(x, y) if x = 3 or y = 0;
        """.encode(
                "ascii"
            )
        )
        f.flush()

        oso.load_file(f.name)
        query = oso._query(Predicate("f", (1, 0)))

        results = [r for r in query.run()]

        trace = query.trace()
        with open("trace.json", "w") as fw:
            json.dump(trace, fw)


def test_graph():
    build_trace_file()
    event_map = {
        "ChoicePush": "blue",
        "ExecuteGoal": "yellow",
        "ExecuteChoice": "orange",
        "Bindings": "purple",
        "Backtrack": "red",
        "Result": "green",
        "Done": "black",
    }
    dot = Digraph(comment="Trace graph")
    with open("trace.json") as f:
        data = json.load(f)
        for node in data:
            event_type = node["event_type"]
            color = event_map[event_type]
            name = str(node["id"])
            label = event_type
            if event_type == "ExecuteGoal":
                label = node["goal"]["polar"]
            if event_type != "Bindings":
                dot.node(name, label=label, color=color)

        for node in data:
            if node["event_type"] != "Bindings":
                parent_id = node["parent_id"]
                id = node["id"]
                if parent_id != id:
                    dot.edge(str(parent_id), str(id))

    dot.render("trace.gv", view=True)
