# Configuration file for the Sphinx documentation builder.
#
# This file only contains a selection of the most common options. For a full
# list see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Path setup --------------------------------------------------------------

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
import os
import sys
from sphinx.highlighting import lexers

sys.path.insert(0, os.path.abspath(".."))
sys.path.insert(0, os.path.abspath("."))
import lexer


# -- Project information -----------------------------------------------------

project = "oso"
copyright = "2020, oso"
author = "oso"


# -- General configuration ---------------------------------------------------

master_doc = "contents"

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.doctest",
    "recommonmark",
    "sphinx.ext.todo",
    "sphinx.ext.githubpages",
    "sphinxcontrib.contentui",
]

html_static_path = ["_static"]

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# Don't copy the source or show a link
html_copy_source = False
html_show_sourcelink = False

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = "borland"

# -- Options for HTML output -------------------------------------------------

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
#html_static_path = ["_static"]

html_extra_path = ["_api_docs"]

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = "sphinx_rtd_theme"
html_theme_options = {"logo_only": True}
html_logo = "oso_logo_resized.png"
html_css_files = [
    "css/custom.css",
]

# --- doctest options ----

doctest_test_doctest_blocks = ""

lexers["polar"] = lexer.PolarLexer()
