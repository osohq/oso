from docutils import nodes


def email_role(typ, rawtext, text, lineno, inliner, options={}, content=[]):
    """
    Role to insert e-mail addresses.
    """
    mailto = """<a href="mailto:%s">%s</a>""" % (text, text)
    node = nodes.raw("", mailto, format="html")
    return [node], []


def setup(app):
    app.add_role("email", email_role)

    return {
        "version": (0, 0, 1),
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }
