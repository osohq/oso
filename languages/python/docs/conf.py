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
from sphinxcontrib.spelling.filters import ContractionFilter

from enchant.tokenize import Filter

sys.path.insert(0, os.path.abspath("../django-oso"))
sys.path.insert(0, os.path.abspath("../flask-oso"))
sys.path.insert(0, os.path.abspath("../sqlalchemy-oso"))
sys.path.insert(0, os.path.abspath(".."))
sys.path.insert(0, os.path.abspath("."))
sys.path.append(os.path.abspath("_ext"))

# DJANGO SETUP FOR DJANGO-OSO #

import django
from django.conf import settings

settings.configure()
django.setup()

##

import lexer

# -- Project information -----------------------------------------------------

project = "oso"
copyright = "2020-2021 Oso Security, Inc"
author = "oso"
version = "0.26.4"
release = "0.26.4"


# -- General configuration ---------------------------------------------------

html_title = "oso Documentation"
release_mode = os.environ.get("DOCS_RELEASE", "") == "1"

master_doc = "index"

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    "sphinx_copybutton",
    "sphinx.ext.autodoc",
    "sphinx.ext.doctest",
    "sphinx.ext.extlinks",
    "sphinx.ext.githubpages",
    "sphinx.ext.ifconfig",
    "sphinx.ext.todo",
    "sphinxcontrib.contentui",
    "sphinxcontrib.spelling",
]


lexers["node"] = lexer.NodeShellLexer()
lexers["polar"] = lexer.PolarLexer()
lexers["jshell"] = lexer.JShellLexer()
lexers["oso"] = lexer.OsoLexer()


class HyphenatedWordFilter(Filter):
    """Treat some hyphenated words as allowed due to made up words in our docs."""

    # This cannot just be allowed words because hyphenated words are split.
    words = {"un-taken", "un-run"}

    def _skip(self, word):
        return word in self.words


spelling_word_list_filename = "spelling_allowed_words.txt"
spelling_filters = [
    # Fix spell check of contractions
    ContractionFilter,
    HyphenatedWordFilter,
]

# html_static_path = ["_static"]

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = ["_build"]

# Don't copy the source or show a link
html_copy_source = False
html_show_sourcelink = False

# add copy button to <pre> elements inside a div with class="copyable"
copybutton_selector = "div.copybutton pre"
copybutton_prompt_text = "\\[\\d*\\]: |\\.\\.\\.: |\\$ "
copybutton_prompt_is_regexp = True

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = "borland"


### Show/hide TODOs

todo_include_todos = False

# -- Options for HTML output -------------------------------------------------

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
# html_static_path = ["_static"]

# html_extra_path = ["_api_docs"]

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
# html_theme = "sphinx_rtd_theme"
html_theme = "sphinx_material"
# html_theme_options = {"logo_only": True}
html_theme_options = {
    # Include the master document at the top of the page in the breadcrumb bar.
    "master_doc": False,
    # Set the name of the project to appear in the navigation.
    "nav_title": "oso Documentation",
    # Specify a base_url used to generate sitemap.xml. If not
    # specified, then no sitemap will be built.
    "base_url": "https://docs.osohq.com/",
    # Set the color and the accent color
    "color_primary": "#0E024E",
    "color_accent": "#FFFFFF",
    # Set the repo location to get a badge with stats
    "repo_url": "https://github.com/osohq/oso/",
    "repo_name": "osohq/oso",
    # Visible levels of the global TOC; -1 means unlimited
    "globaltoc_depth": 3,
    # If False, expand all TOC entries
    "globaltoc_collapse": True,
    # If True, show hidden TOC entries
    "globaltoc_includehidden": True,
    # "heroes": {"index": "Welcome to the home of the oso documentation!",},
    "html_minify": release_mode,
    "css_minify": release_mode,
    "nav_links": False,
}
html_show_sphinx = False
version_dropdown = False
html_sidebars = {"**": ["globaltoc.html", "localtoc.html"]}

# html_logo = "oso_logo_trimmed.png"
html_js_files = [
    # "js/custom.js",
]
html_css_files = [
    # "css/custom.css",
    # "css/matter.css",
]

# html_favicon = "favicon.ico"
