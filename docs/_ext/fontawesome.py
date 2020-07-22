import os
from docutils.nodes import strong, emphasis, reference, Text
from docutils.parsers.rst.roles import set_classes
from docutils.parsers.rst import Directive
import docutils.parsers.rst.directives as directives

# add role
def fa_global(key="", style="fa"):
    def fa(role, rawtext, text, lineno, inliner, options={}, content=[]):
        options.update({"classes": []})
        options["classes"].append(style)
        if key:
            options["classes"].append("fa-%s" % key)
        else:
            for x in text.split(","):
                options["classes"].append("fa-%s" % x)
        set_classes(options)
        node = emphasis(**options)
        return [node], []

    return fa


def setup(app):
    app.add_role("fa", fa_global())
    app.add_role("fas", fa_global(style="fas"))
    app.add_role("fab", fa_global(style="fab"))

    return {"version": (0, 0, 1)}
