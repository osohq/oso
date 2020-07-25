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

sys.path.insert(0, os.path.abspath(".."))
sys.path.insert(0, os.path.abspath("."))
import lexer


# -- Project information -----------------------------------------------------

project = "oso"
copyright = "2020, oso"
author = "oso"


# -- General configuration ---------------------------------------------------

master_doc = "index"

sys.path.append(os.path.abspath("./_ext"))

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
    "sphinx_tabs.tabs",
    "sphinx.ext.ifconfig",
    "sphinxcontrib.spelling",
    "sphinx_copybutton",
    "fontawesome",
]


class HyphenatedWordFilter(Filter):
    """Treat some hypthenated words as allowed due to made up words in our docs."""

    # This cannot just be allowed words because hypthenated words are split.

    words = {"un-taken", "un-run"}

    def _skip(self, word):
        return word in self.words


spelling_word_list_filename = "spelling_allowed_words.txt"
spelling_filters = [
    # Fix spell check of contractions
    ContractionFilter,
    HyphenatedWordFilter,
]

html_static_path = ["_static"]

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = [
    "**.tmp",
    "_build",
    "Thumbs.db",
    ".DS_Store",
    "theme/**",
    "project/changelogs/vNEXT.rst",
    "project/changelogs/vTEMPLATE.rst",
    "**.pytest_cache**",
    "ruby/README.md",
    "more/language/polar-classes.rst",  # we don't currently have classes
    "**/venv/**",
]

# Don't copy the source or show a link
html_copy_source = False
html_show_sourcelink = False

# add copy button to <pre> elements inside a div with class="copyable"
copybutton_selector = "div.copybutton pre"
copybutton_prompt_text = "\\[\\d*\\]: |\\.\\.\\.: "
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

html_extra_path = ["_api_docs"]

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
# html_theme = "sphinx_rtd_theme"
html_theme = "sphinx_material"
# html_theme_options = {"logo_only": True}
html_theme_options = {
    # Set the name of the project to appear in the navigation.
    "nav_title": "Documentation",
    # Set you GA account ID to enable tracking
    "google_analytics_account": "UA-XXXXX",
    # Specify a base_url used to generate sitemap.xml. If not
    # specified, then no sitemap will be built.
    "base_url": "https://docs.osohq.com/",
    # Set the color and the accent color
    "color_primary": "#0E024E",
    "color_accent": "#0E024E",
    # Set the repo location to get a badge with stats
    "repo_url": "https://github.com/osohq/oso/",
    "repo_name": "oso",
    # Visible levels of the global TOC; -1 means unlimited
    "globaltoc_depth": 3,
    # If False, expand all TOC entries
    "globaltoc_collapse": True,
    # If True, show hidden TOC entries
    "globaltoc_includehidden": True,
    # "heroes": {"index": "Welcome to the home of the oso documentation!",},
}

version_dropdown = True
version_info = {"release": "/", "devel": "/devel"}
html_sidebars = {
    "**": ["logo-text.html", "globaltoc.html", "localtoc.html", "searchbox.html"]
}

html_logo = "oso_logo_resized.png"
html_js_files = [
    "js/custom.js",
]
html_css_files = [
    "css/custom.css",
]

html_favicon = "favicon.ico"

# --- doctest options ----

doctest_test_doctest_blocks = ""

lexers["polar"] = lexer.PolarLexer()
