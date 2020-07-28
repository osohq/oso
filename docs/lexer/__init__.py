from pygments.lexers.jvm import JavaLexer
from pygments.lexer import Lexer, RegexLexer, bygroups, do_insertions
from pygments import token
import re

line_re = re.compile(".*?\n")


class PolarLexer(RegexLexer):
    tokens = {
        "root": [
            (r"(\w[\w-]*)(\()", bygroups(token.Keyword, token.Punctuation)),
            (
                r"\sif\s|\sand\s|\sor\s|\snot\s|\smatches\s|\strue\s|\sfalse\s",
                token.Name.Function,
            ),
            (r"\w[\w-]*", token.Text),
            (r"\s", token.Text),
            (r"=|\?=|\*|<|<=|>|>=|==", token.Operator),
            (r",|\.|\(|\)|\{|\}|:|;|\[|\]|\^", token.Punctuation),
            (r"\#.*$", token.Comment.Single),
            (r'"', token.String, "string"),
        ],
        "string": [(r'[^"]+', token.String), (r'"', token.String, "#pop"),],
    }


class JShellLexer(Lexer):
    name = "JShell session"
    aliases = ["jshell"]
    mimetypes = ["text/x-java-doctest"]

    def get_tokens_unprocessed(self, text):
        javalexer = JavaLexer(**self.options)

        curcode = ""
        insertions = []

        prompt = u"jshell> "
        prompt_len = len(prompt)

        for match in line_re.finditer(text):
            line = match.group()
            if line.startswith(prompt):
                insertions.append(
                    (len(curcode), [(0, token.Generic.Prompt, line[:prompt_len])])
                )
                curcode += line[prompt_len:]
            else:
                if curcode:
                    for item in do_insertions(
                        insertions, javalexer.get_tokens_unprocessed(curcode)
                    ):
                        yield item
                    curcode = ""
                    insertions = []
                yield match.start(), token.Generic.Output, line
        if curcode:
            for item in do_insertions(
                insertions, javalexer.get_tokens_unprocessed(curcode)
            ):
                yield item


class OsoLexer(Lexer):
    name = "oso session"
    aliases = ["oso"]
    mimetypes = ["text/x-polar-doctest"]

    def get_tokens_unprocessed(self, text):
        polarlexer = PolarLexer(**self.options)

        curcode = ""
        insertions = []

        qprompt = u"query> "
        rprompt = u"debug> "

        prompt_len = len(qprompt)

        for match in line_re.finditer(text):
            line = match.group()
            if line.startswith(qprompt) or line.startswith(rprompt):
                insertions.append(
                    (len(curcode), [(0, token.Generic.Prompt, line[:prompt_len])])
                )
                curcode += line[prompt_len:]
            else:
                if curcode:
                    for item in do_insertions(
                        insertions, polarlexer.get_tokens_unprocessed(curcode)
                    ):
                        yield item
                    curcode = ""
                    insertions = []
                yield match.start(), token.Generic.Output, line
        if curcode:
            for item in do_insertions(
                insertions, polarlexer.get_tokens_unprocessed(curcode)
            ):
                yield item
