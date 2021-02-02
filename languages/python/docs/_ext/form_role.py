from docutils import nodes
import jinja2
from docutils.parsers.rst.directives import unchanged
from sphinx.util.docutils import SphinxDirective

FORM_TEMPLATE = jinja2.Template(
    u"""
<div class="subscribe-form align-right newsletter-footer w-form">
    <form id="subscribe-form-docs" name="subscribe-form-docs">
        <label class="matter-textfield-filled">
            <input type="email" maxlength="256" name="Email-Address" id="Email-Address" required=True
                placeholder="Email Address">
        </label>
        <button type="submit" data-wait="Churning..." class="matter-button-contained subscribe">
            Subscribe
        </button
    </form>
</div>
"""
)

# placeholder node for document graph
class form_node(nodes.General, nodes.Element):
    pass


class FormDirective(SphinxDirective):
    required_arguments = 0

    option_spec = {"text": unchanged, "link": unchanged, "class": unchanged}

    # this will execute when your directive is encountered
    # it will insert a form_node into the document that will
    # get visisted during the build phase
    def run(self):
        node = form_node()
        return [node]


# build phase visitor emits HTML to append to output
def html_visit_form_node(self, node):
    html = FORM_TEMPLATE.render()
    self.body.append(html)
    raise nodes.SkipNode


# if you want to be pedantic, define text, latex, manpage visitors too..


def setup(app):
    app.add_node(form_node, html=(html_visit_form_node, None))
    app.add_directive("form", FormDirective)

    return {
        "version": (0, 0, 1),
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }
