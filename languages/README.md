# oso host language interface checklist

Each host language interface should have a column in each table below.
Please keep the tables up to date as languages, fields, methods, etc. are
added, modified, etc. Names marked as `code` are literal, and should agree
(modulo case, spelling, etc.) across implementations; bare names like
"delete" have language-specific names, but should have similar semantics.

## Oso

Top-level authorization API.

| Method       | Python | Ruby | Java |
|--------------|--------|------|------|
| `allow`      | x      | x    | x    |
+ `Polar` methods

## Polar

Top-level Polar language API: load and query Polar.

| Field Name   | Python | Ruby | Java |
|--------------|--------|------|------|
| `ffi_polar`  | x      | x    | x    |
| `host`       | x      | x    | x    |
| `load_queue` | x      | x    | x    |

| Method             | Python | Ruby | Java |
|--------------------|--------|------|------|
| delete             | x      | x    | x    |
| `clear`            | x      | x    | x    |
| `load_file`        | x      | x    | x    |
| `load_str`         | x      | x    | x    |
| `query(str)`       | x      | x    | x    |
| `query(pred)`      | x      | x    | x    |
| `repl`             | x      | x    | x    |
| `register_class`   | x      | x    | x    |
| `register_constant`| x      | x    | x    |
| `load_queued_files`| x      | x    | x    |

## Query

Execute a Polar query through the FFI event interface.

| Class        | Python | Ruby | Java |
|--------------|--------|------|------|
| `Query`      | x      | x    | x    |
| `QueryResult`| x      |      |      |

### Query.Query

| Field Name   | Python | Ruby | Java |
|--------------|--------|------|------|
| `ffi_query`  | x      | x    | x    |
| `host`       | x      | x    | x    |
| `calls`      | x      | x    | x    |

| Event Name                 | Python | Ruby | Java |
|----------------------------|--------|------|------|
| `Debug`                    | x      | x    | x    |
| `Done`                     | x      | x    | x    |
| `ExternalCall`             | x      | x    | x    |
| `ExternalIsa`              | x      | x    | x    |
| `ExternalIsSubSpecializer` | x      | x    | x    |
| `ExternalOp`               | x      |      |      |
| `ExternalUnify`            | x      | x    |      |
| `MakeExternal`             | x      | x    | x    |
| `Result`                   | x      | x    | x    |

## Host

Maintain mappings & caches for host language classes & instances.

| Field Name         | Python | Ruby | Java |
|--------------------|--------|------|------|
| `ffi_polar`        | x      | x    | x    |
| `classes`          | x      | x    | x    |
| `constructors`     | x      | x    | x    |
| `instances`        | x      | x    | x    |

| Method             | Python | Ruby | Java |
|--------------------|--------|------|------|
| delete             | x      | x    | x    |
| copy               | x      | x    | x    |
| `get_class`        | x      | x    | x    |
| `cache_class`      | x      | x    | x    |
| `get_constructor`  | x      | x    |      |
| `get_instance`     | x      | x    | x    |
| `has_instance`     |        | x    | x    |
| `cache_instance`   | x      | x    | x    |
| `make_instance`    | x      | x    | x    |
| `isa`              | x      | x    | x    |
| `is_subspecializer`| x      | x    | x    |
| `unify`            | x      | x    |      |
| `operator`         | x      |      |      |
| `to_polar_term`    | x      | x    | x    |
| to_host            | x      | x    | x    |
