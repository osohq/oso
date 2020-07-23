from pygments.lexer import RegexLexer, bygroups
from pygments import token


class PolarLexer(RegexLexer):
    tokens = {
        "root": [
            (r"(\w[\w-]*)(\()", bygroups(token.Keyword, token.Punctuation)),
            (r"\w[\w-]*", token.Name),
            (r"\s", token.Text),
            (r"=|\||:=|\?=|\!|\*|<|<=|>|>=|==", token.Operator),
            (r"if|and|or|not", token.Operator.Word),
            (r",|\.|\(|\)|\{|\}|:|;|\[|\]", token.Punctuation),
            (r"\#.*$", token.Comment.Single),
            (r'"', token.String, "string"),
        ],
        "string": [(r'[^"]+', token.String), (r'"', token.String, "#pop"),],
    }
