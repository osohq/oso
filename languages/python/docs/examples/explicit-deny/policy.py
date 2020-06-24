import pathlib

import oso
from polar import Variable


def setup(oso):
    oso.load_file(pathlib.Path(__file__).parent / "policy.polar")

def auth(oso, actor, action, resource):
    results = oso.query_predicate(
        "decide", actor, action, resource, Variable("result"), Variable("reason"))

    try:
        result = results.results[0]
        return result['result'] == 'allow', result['reason']
    except IndexError:
        return False, "No result"
