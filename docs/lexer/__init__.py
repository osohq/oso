from pygments.lexer import RegexLexer, bygroups
from pygments import token


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
            (r",|\.|\(|\)|\{|\}|:|;|\[|\]", token.Punctuation),
            (r"\#.*$", token.Comment.Single),
            (r'"', token.String, "string"),
        ],
        "string": [(r'[^"]+', token.String), (r'"', token.String, "#pop"),],
    }
