from pygments.lexer import Lexer, RegexLexer, bygroups, do_insertions
from pygments.lexers.jvm import JavaLexer
from pygments.lexers.javascript import JavascriptLexer
from pygments import token
import re

line_re = re.compile(".*?\n")


class PolarLexer(RegexLexer):
    tokens = {
        "root": [
            (r"(:)(\s\b[A-Z].*?\b)", bygroups(token.Punctuation, token.Name.Class)),
            (r"(\w[\w-]*)(\()", bygroups(token.Keyword, token.Punctuation)),
            (
                r"\sif\s|\sand\s|\sor\s|\snot\s|\smatches\s|\strue\s|\sfalse\s|\sin\s",
                token.Keyword,
            ),
            (r"\w[\w-]*\??", token.Text),
            (r"\s", token.Text),
            (r"=|\?=|\*|<|<=|>|>=|==|!=|-", token.Operator),
            (r",|\.|\(|\)|\{|\}|:|;|\[|\]|\^", token.Punctuation),
            (r"\#.*$", token.Comment.Single),
            (r'"', token.String, "string"),
        ],
        "string": [
            (r'[^"]+', token.String),
            (r'"', token.String, "#pop"),
        ],
    }


class GenericShellLexer(Lexer):
    lexer_class = None
    prompts = []

    def get_tokens_unprocessed(self, text):
        assert self.lexer_class
        lexer = self.lexer_class(**self.options)

        curcode = ""
        insertions = []

        for match in line_re.finditer(text):
            line = match.group()
            prompt = None
            for p in self.prompts:
                if line.startswith(p):
                    prompt = p
            if prompt:
                prompt_len = len(prompt)
                insertions.append(
                    (len(curcode), [(0, token.Generic.Prompt, line[:prompt_len])])
                )
                curcode += line[prompt_len:]
            else:
                if curcode:
                    yield from do_insertions(
                        insertions, lexer.get_tokens_unprocessed(curcode)
                    )

                    curcode = ""
                    insertions = []
                yield match.start(), token.Generic.Output, line
        if curcode:
            yield from do_insertions(insertions, lexer.get_tokens_unprocessed(curcode))


class JShellLexer(GenericShellLexer):
    name = "JShell session"
    aliases = ["jshell"]
    mimetypes = ["text/x-java-doctest"]

    lexer_class = JavaLexer
    prompts = ["jshell> "]


class OsoLexer(GenericShellLexer):
    name = "oso session"
    aliases = ["oso"]
    mimetypes = ["text/x-polar-doctest"]

    lexer_class = PolarLexer
    prompts = ["query> ", "debug> "]


class NodeShellLexer(GenericShellLexer):
    name = "Node REPL"
    aliases = ["node"]
    mimetypes = ["text/x-javascript-doctest"]

    lexer_class = JavascriptLexer
    prompts = ["> "]
