from docutils import nodes
import jinja2
from docutils.parsers.rst.directives import unchanged
from sphinx.util.docutils import SphinxDirective

BUTTON_TEMPLATE = jinja2.Template(
    u"""
<a href="{{ link }}">
    <button class="matter-button-contained {{ button_class }} ">{{ text }}</button>
</a>
"""
)

# placeholder node for document graph
class button_node(nodes.General, nodes.Element):
    pass


class ButtonDirective(SphinxDirective):
    required_arguments = 0

    option_spec = {"text": unchanged, "link": unchanged, "class": unchanged}

    # this will execute when your directive is encountered
    # it will insert a button_node into the document that will
    # get visisted during the build phase
    def run(self):
        env = self.state.document.settings.env
        app = env.app

        node = button_node()
        node["text"] = self.options["text"]
        node["link"] = self.env.doc2path(self.options["link"])
        node["class"] = self.options.get("class", "")
        return [node]


# build phase visitor emits HTML to append to output
def html_visit_button_node(self, node):
    html = BUTTON_TEMPLATE.render(
        text=node["text"],
        link=node["link"].replace(".rst", ".html"),
        button_class=node["class"],
    )
    self.body.append(html)
    raise nodes.SkipNode


# if you want to be pedantic, define text, latex, manpage visitors too..


def setup(app):
    app.add_node(button_node, html=(html_visit_button_node, None))
    app.add_directive("button", ButtonDirective)

    return {
        "version": (0, 0, 1),
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }
