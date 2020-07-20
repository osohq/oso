"""
Custom Sphinx directive to make it more convenient to do literal diffs
"""

from difflib import unified_diff
from docutils.nodes import literal_block, Text
from sphinx.directives.code import LiteralInclude, LiteralIncludeReader
from docutils.parsers.rst import directives
import re

from typing import List, Tuple


class LiteralIncludeDiffReader(LiteralInclude):
    def show_diff(self, location: Tuple[str, int] = None) -> List[str]:
        print("Called show diff")
        new_lines = self.read_file(self.filename)
        old_filename = self.options.get("diff")
        old_lines = self.read_file(old_filename)
        old_name = self.options.get("caption") or "Original"
        new_name = self.options.get("caption") or "Current"
        diff = unified_diff(old_lines, new_lines, old_name, new_name)
        return list(diff)


class LiteralIncludeDiff(LiteralInclude):
    option_spec = {"base_path": directives.unchanged, **LiteralInclude.option_spec}

    def run(self):
        base_path = self.options.get("base_path")
        if base_path:
            if "diff" in self.options:
                self.options["diff"] = base_path + self.options["diff"]
            self.arguments[0] = base_path + self.arguments[0]
        LiteralIncludeReader.show_diff = LiteralIncludeDiffReader.show_diff
        return super().run()


def setup(app):
    app.add_directive("literalinclude", LiteralIncludeDiff, override=True)

    return {
        "version": "0.1",
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }
