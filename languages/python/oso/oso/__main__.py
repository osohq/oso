import sys

from .oso import Oso

Oso().repl(files=sys.argv[1:])
