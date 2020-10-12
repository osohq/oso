import threading
import random
import time
import multiprocessing
from concurrent.futures import ThreadPoolExecutor
import concurrent.futures

import pytest

from oso import Oso, OsoError

ITERS = 100


class X:
    def __init__(self, x):
        self.x = x

    def __eq__(self, other):
        return self.x == other.x


def r():
    return random.randint(0, 5)


def exhaust(i):
    try:
        for _ in i:
            pass
    except OsoError:
        pass


def torch_oso(oso):
    for i in range(ITERS):
        x = X(r())
        y = X(r())
        exhaust(oso.query_rule("allow", x, y))
        time.sleep(random.random() * 0.01)

        exhaust(oso.query_rule("allow", r(), r()))

        exhaust(oso.query_rule("allow", 1))
        time.sleep(random.random() * 0.01)

        exhaust(oso.query_rule("allow", r(), str(r())))


def test_multi():
    oso = Oso()
    oso.load_str("allow(x, y) if x == y;")

    tp = ThreadPoolExecutor(max_workers=8)

    futures = []
    for _ in range(32):
        futures.append(tp.submit(torch_oso, oso))

    for i, future in enumerate(concurrent.futures.as_completed(futures)):
        future.result()

    # If we got here none of these crashed.
    assert True
