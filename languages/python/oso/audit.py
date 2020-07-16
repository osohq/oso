import io
import os
import pickle
import logging
import json
from contextlib import closing, contextmanager
from datetime import datetime, timezone
from sqlite3 import Connection, IntegrityError
from polar import Predicate

from typing import Any, Dict

logger = logging.getLogger(__name__)

ENABLE_AUDIT = False


def enable():
    global ENABLE_AUDIT
    ENABLE_AUDIT = True


class Unknown:
    fields: Dict[str, Any] = {}

    def __setstate__(self, dict):
        self.fields = dict

    def __str__(self):
        return f"{self.__class__.__name__}{{{', '.join(f'{k}: {v}' for k, v in self.fields.items())}}}"


class LenientUnpickler(pickle.Unpickler):
    """Attempts to unpickle, but falls back to an unknown class if it raises an exception"""

    def find_class(self, module, name):
        try:
            return super(LenientUnpickler, self).find_class(module, name)
        except Exception as e:
            logger.error(f"Could not find class {name} in module {module}")
            logger.exception(e)
            cls = type(name, (Unknown,), {})
            return cls


def load(s):
    return LenientUnpickler(io.BytesIO(s)).load()


class AuditEntry:
    """class to store audit logs"""

    def __init__(self, row: tuple):
        self.id = row[0]
        self.timestamp = row[1]
        self.actor = load(row[2])
        self.action = load(row[3])
        self.resource = load(row[4])
        self.success = row[5] == 1
        trace = row[6]
        self.trace = json.loads(trace) if trace else None


class AuditLog:
    def __init__(self):
        self.db_path = os.getenv("DB_PATH", "audit.db")
        print("db path")
        print(self.db_path)
        if not self.db_path:
            raise ValueError(
                "Please initialize the AuditLog with the path to a SQLite3 DB."
            )
        with self._cursor() as cur:
            print("creating table")
            cur.execute(
                "CREATE TABLE IF NOT EXISTS events ( "
                "id INTEGER NOT NULL PRIMARY KEY, "
                "timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, "
                "actor BLOB NOT NULL, "
                "action BLOB NOT NULL, "
                "resource BLOB NOT NULL, "
                "success BOOLEAN NOT NULL CHECK (success IN (0,1)), "
                "trace BLOB "
                ");"
            )

    @contextmanager
    def _cursor(self):
        with closing(Connection(self.db_path)) as conn:
            with conn:
                with closing(conn.cursor()) as cur:
                    yield cur

    def write(self, event: tuple):
        with self._cursor() as cur:
            cur.execute(
                "INSERT INTO events (timestamp, actor, action, resource, success, trace) "
                "VALUES (?, ?, ?, ?, ?, ?)",
                event,
            )

    def iter(self):
        with self._cursor() as cur:
            for row in cur.execute("SELECT * FROM events"):
                yield AuditEntry(row)

    def get(self, id: int = 0):
        with self._cursor() as cur:
            result = cur.execute("SELECT * FROM events WHERE id = ?", (id,)).fetchone()
            if not result:
                return None
            return AuditEntry(result)

    def count(self):
        with self._cursor() as cur:
            return cur.execute("SELECT count(*) FROM events").fetchone()[0] + 1

    def clear(self):
        with self._cursor() as cur:
            cur.execute("DELETE from events")


def log(actor, action, resource, result):
    if ENABLE_AUDIT:
        try:
            timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")
            actor = pickle.dumps(actor)
            action = pickle.dumps(action)
            resource = pickle.dumps(resource)
            success = 1 if result.success else 0
            trace = None
            if success:
                trace = json.dumps(result.traces[0])
            AuditLog().write((timestamp, actor, action, resource, success, trace))
        except ValueError:
            logger.debug("no audit DB configured; cannot log event")
        except Exception as e:
            logger.exception(e)
