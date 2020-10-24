"""Set of test helpers to match test helpers from Python Polar."""
import pytest

from polar import Polar


# DEFINED So pytests have same interface.
@pytest.fixture
def db():
    """ Set up the polar database """
    raise NotImplementedError()


@pytest.fixture
def polar():
    """ Set up a polar instance and tear it down after the test."""
    p = Polar()
    yield p
    del p


@pytest.fixture
def tell(polar):
    """ Define a fact or rule in the polar database """

    def _tell(f):
        # TODO (dhatch): Temporary until rewritten parser supports optional
        # semicolon.
        if not f.endswith(";"):
            f += ";"

        polar.load_str(f)

    return _tell


@pytest.fixture
def load_file(polar):
    """ Load a source file """

    def _load_file(f):
        polar.load_file(f)

    return _load_file


@pytest.fixture
def query(polar):
    """ Query something and return the results as a list """

    def _query(q):
        return list(r["bindings"] for r in polar.query(q))

    return _query


@pytest.fixture
def qeval(polar, query):
    """ Query something and return if there's exactly 1 result """

    def _qeval(q):
        result = list(query(q))
        return len(result) == 1

    return _qeval


@pytest.fixture
def qvar(polar, query):
    """ Query something and pull out the results for the variable v """

    def _qvar(q, v, one=False):
        results = query(q)
        if one:
            assert len(results) == 1, "expected one result"
            return results[0][v]
        return [env[v] for env in results]

    return _qvar


private_key = """-----BEGIN RSA PRIVATE KEY-----
MIIJKgIBAAKCAgEA5iR6CAsP8i6Fzt3mqBO39rwV58Qoe6Kgh/h+6qQDanNGllif
lUP1TqZJ0kt/Wiqm0uqURt8Oe6i9VgcRsfTw75pNNMV5FRZaL6gGxiM1JaaY6dni
N1Vhr8JntOep7yCkn1lEae3fYdrA+sCHaavTYyX6uaO67AVDYvLb/0+CpjXxblWW
TDDrFJ4+zQIkftYlELv4awirSkkz1FmPShTFz8fkP+uHX7GKBCyV3+Z6AI9FqXju
XqNzBWoB5vJvF4+OjN2SQTSSPZvkijaRsktByu/DLpepUT1ybkT98DBHRYSOfPtx
lqDi9M2Uv8t4/RQjL0cj3G209y8n3eW3SzhpedJxZJIVPK7zWbIyJguplMOGUWza
YDuoEyXN6eVGwG8L98LKKIyIAK31d5hHRZlHPfYdic6VC19Izry6WmtgBy/DrK23
aSI8KDTkrOle76prbYoTjlLrkTEjw/0ffd1XYuxd30hR3BEc85yAgtIpkzf7UZxB
O10uePPX3bwtDb9uXdVjXT+IBICpvjptC5HV/EeeK14jO0OUtRaPirLygcOUi9qQ
uRxCG72s1O0Oi9XJahyguo9o4HDP5PTo9WaJN23ox3q7Rf0votBPi2z6UN/gIclA
dfGFron0DRoGAwKzioRN9c2nWZuZ1WiY2a+arS3iDBtla1BFF2CjiXIKuj8CAwEA
AQKCAgEA21u1POleX5XcUFrNOTLiS7jmoCwl8gIGRNOkFP4Ti2koxLDgGqPVswto
nZr8XfL9Y1fX0N1WrqMdJFxEj3xKEfbe1AfM6z6M45OiMUTpqWNrqKnWpqspGx+P
Paz6GkTg5Elvng/utRSOj/Lmnt/58i0HF95pkgFKQ4v8CRO+EbKk1meZhDG0P8i9
TyZVptdyKMshctOmgH3ZevOKUjAOg4ehlRNnytwsEuJ0UB8b9mCZI3lyqp3cSjLK
cVhubuSUGMwwVRpIsZRfdyWgJXL52PZC8av45MhOw6/a4w8BP8+jCJmZoNrkuDUj
LNRCE+cXyj5iff3LWTeHJkeIN5gjXv1GEyvxV/NjR5K2wkhr8RDEL+Ert09hrRgj
g2S3V14woEriq8Zh6pWxVyMjt+vdrnMd9Mo0VzIMEYEm9Fpe2oK+GfjrrDcwi0ho
BHyElNGfdpeptoeGwXd2299O4eCluztcN62e6ZqKP1S/pH+Hd+V9Bv03kThGAYg9
cK6555Aot7sRnE7qn4d1OqogHFmZ/PcETp0oSlOnWi7//GDe8Cu6pz8kLhZsVodw
o/adNx1XS16LW2yDe84Hw0AglDiR+XGR5LUGcYtMbRoIBcYek9D0aTAsh3iu5DdK
j0A3HkXkita5WyLDHfIggpzUjW8cG9szO5RuJwdGDUy1zkfiwvECggEBAPlvNIfX
JrrOfL6H1kzxm4ABmqX9aOuatc6e5ZpZUmXPJesDUJBzNrP1oSKnFVF991krlaux
iEuwYxdTwmThQHlu1hPxDeo/DRMFE+bBriBmh/jNW6iV+4Nqh110XGPMYJvK/OT1
tnQgfioS2Rl/wALKE/RBC2z1Lo7UTAWcEEw/8kO1Wlmr5f0rg+VPAEOWQXfQ2Y7P
dbrEnFsmJ02wjb9H1QaRDTZEmQVNgt7jw+ydZa2rI7vTMlIa3ytW3axNw/Of5TB1
1wQ/qdoyQQ/Gemm2S1YzJ09p7OXAQipZIUgxZiBrn6aX3v3C37MSOVPqH4/nuxkz
v6Vp8W3trkRfqBsCggEBAOwzRkCZbXKo1JEeLkYl11PXM1MbMQ1gz5EyGb3rt2eN
446zi+i22GuaQ4k2NlTDSocXejiz+7PGCzfLu2M9zJ2Gwo5gAKLXws/DoLJAd3Rh
GySjGIWTfQwe8PHzG3GxQRQo8NsEg3kee5kzqvyrZq7IdJJPhNzPPuCnTlqdt8vR
EjhoFwKE3892g7W3t7YOQwQwRuT2UVVdcflC7trSflRNSsiCdy0iQF7raradq/ma
qqZr+pxYVA8rHKfsxuGO27KQdmV4dlmsEC4Zjb8fyAFDvOgn7CoJeYDuBbGHO8Ea
abgFHVvca5qPKfsJB+4UjeHZJGWJRLKY5vBUMPjUYK0CggEAUIgF1sGxAGkAP2eN
2eO7h7V835CUWlTl0+LbUFz8TGB35ot4bFq2U52/O3fkWx4nSMPYm8lCruUw6Owi
+/z5mvsc5O1Tx8g5iWV/SmZHuLBBwCNVL3XU8VXohFS4K4RlwIbl5WorUQzYju7s
5t2m+X+St650aOYz9Os37Cu521Rd/FxF4mOsanOtLtC1zhxp4KwuQXxbj0RBEvCb
ieqxqQshDPCx6k51dQ/Ua+/vZqpelJaHf/Gs8nM4kD6IbUPiOvrpvR6eoAGJ7ieB
d/1lslGnuxni3DHfyUGsWw3RwAQq69azgc7QsO9E2ATPO3eAXy666mUZv+cXip1N
QUf89wKCAQEAnPQxgamyZde0cL4KZ8irfmXpEBdokAg9xbDyFBb3Z5OMm/3JQZmG
1HHM4PeqQMcI1h4OtHE9F6fJOolh+r9NIXwz+mHm6k7PgDnxpaFa/3WrkLvkBpcM
KCrDVzOBkBoGMbxG2HL2XlyYKyR/Qakv8YL4m2TF1+jLUoM6eNKHGKPUJLFeYOkk
w/pv6SespwhxFe5ynaDkSQJwQv9sMvJeyewWfojbYp15AtoSrki1x4Y0UaQ9Avla
2j1+rEOVoLrKWKzQT/stQccpdUi7vT4ELHrzo50rvH9RQxBnriE73sSLbaHQcYNV
6X2qmsrUfysfYO1m1yXRBZC/HQIFDMQrfQKCAQEAr5ViGvKV4a+dKaYJ510C1XGd
exhLxgWB2svpFdP/wgguM9DUahKZbWbctnUnKqei/IGwfVn5assWL+cYGDB60LZ2
df9F8D6mcq1uMhFcAzfdBNv8+hpWLBi+s5M3OfHEXDbeM3++88JqY99eKC0jn1YE
AhF1NhuU0ANGb0juX7LbQKxOmqJjv1xC9XSR5WQ0VMfcrRPmsUurgLwh01rwsqQx
mLyRq8fhtrnCKjYDfzA9LIBpFoLWR/YMGizOrzHiF8iJmD1waTeVS1GHaxpxIXAe
/BRmYN+8kjKX7n15en5RbhVMtkdpxNLZ5OSMsEt/YRidcS6Gk5y7MUSrCxOhXQ==
-----END RSA PRIVATE KEY-----"""

public_key = """-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA5iR6CAsP8i6Fzt3mqBO3
9rwV58Qoe6Kgh/h+6qQDanNGlliflUP1TqZJ0kt/Wiqm0uqURt8Oe6i9VgcRsfTw
75pNNMV5FRZaL6gGxiM1JaaY6dniN1Vhr8JntOep7yCkn1lEae3fYdrA+sCHaavT
YyX6uaO67AVDYvLb/0+CpjXxblWWTDDrFJ4+zQIkftYlELv4awirSkkz1FmPShTF
z8fkP+uHX7GKBCyV3+Z6AI9FqXjuXqNzBWoB5vJvF4+OjN2SQTSSPZvkijaRsktB
yu/DLpepUT1ybkT98DBHRYSOfPtxlqDi9M2Uv8t4/RQjL0cj3G209y8n3eW3Szhp
edJxZJIVPK7zWbIyJguplMOGUWzaYDuoEyXN6eVGwG8L98LKKIyIAK31d5hHRZlH
PfYdic6VC19Izry6WmtgBy/DrK23aSI8KDTkrOle76prbYoTjlLrkTEjw/0ffd1X
Yuxd30hR3BEc85yAgtIpkzf7UZxBO10uePPX3bwtDb9uXdVjXT+IBICpvjptC5HV
/EeeK14jO0OUtRaPirLygcOUi9qQuRxCG72s1O0Oi9XJahyguo9o4HDP5PTo9WaJ
N23ox3q7Rf0votBPi2z6UN/gIclAdfGFron0DRoGAwKzioRN9c2nWZuZ1WiY2a+a
rS3iDBtla1BFF2CjiXIKuj8CAwEAAQ==
-----END PUBLIC KEY-----"""
