# oso host language interface checklist

Each host language interface should have a column in each table below.
Please keep the tables up to date as languages, fields, methods, etc. are
added, modified, etc. Names marked as `code` are literal, and should agree
(modulo case, spelling, etc.) across implementations; bare names like
"delete" have language-specific names, but should have similar semantics.

## Oso

Top-level authorization API.

| Method       | Python | Ruby | Java | Node.js |
| ------------ | ------ | ---- | ---- | ------- |
| `is_allowed` | x      | x    | x    | x       |

- `Polar` methods

## Polar

Top-level Polar language API: load and query Polar.

| Field Name        | Python | Ruby | Java | Node.js |
| ----------------- | ------ | ---- | ---- | ------- |
| `ffi_polar`       | x      | x    | x    | x       |
| `host`            | x      | x    | x    | x       |

| Method              | Python | Ruby | Java | Node.js |
| ------------------- | ------ | ---- | ---- | ------- |
| delete              | x      | x    | x    | x       |
| `clear`             | x      | x    | x    | x       |
| `load_file`         | x      | x    | x    | x       |
| `load_str`          | x      | x    | x    | x       |
| `query(str)`        | x      | x    | x    | x       |
| `query(pred)`       | x      | x    | x    | x       |
| `query_rule`        | x      | x    | x    | x       |
| `repl`              | x      | x    | x    |
| `register_class`    | x      | x    | x    | x       |
| `register_constant` | x      | x    | x    | x       |

## Query

Execute a Polar query through the FFI event interface.

| Class         | Python | Ruby | Java | Node.js         |
| ------------- | ------ | ---- | ---- | --------------- |
| `Query`       | x      | x    | x    | x               |
| `QueryResult` | x      |      |      | type, not class |

### Query.Query

| Field Name  | Python | Ruby | Java | Node.js |
| ----------- | ------ | ---- | ---- | ------- |
| `ffi_query` | x      | x    | x    | x       |
| `host`      | x      | x    | x    | x       |
| `calls`     | x      | x    | x    | x       |
| `results`   |        | x    |      | x       |

| Event Name                 | Python | Ruby | Java | Node.js |
| -------------------------- | ------ | ---- | ---- | ------- |
| `Debug`                    | x      | x    | x    | x       |
| `Done`                     | x      | x    | x    | x       |
| `ExternalCall`             | x      | x    | x    | x       |
| `ExternalIsa`              | x      | x    | x    | x       |
| `ExternalIsSubSpecializer` | x      | x    | x    | x       |
| `ExternalOp`               | x      |      |      |         |
| `ExternalUnify`            | x      | x    |      | x       |
| `MakeExternal`             | x      | x    | x    | x       |
| `Result`                   | x      | x    | x    | x       |

| Method              | Python | Ruby    | Java         | Node.js |
| ------------------- | ------ | ------- | ------------ | ------- |
| `question_result`   |        | x       |              | x       |
| `register_call`     |        | x       | x            | x       |
| `call_result`       |        | x       |              | x       |
| `next_call_result`  |        | x       | x            | x       |
| `application_error` |        | x       |              | x       |
| `handle_call`       |        | x       | x            | x       |
| `has_more_elements` |        |         | x            |         |
| `next_element`      |        |         | x            |         |
| `results`           |        |         | x            |         |
| `getCall`           |        |         | x            |         |
| `run`               | `run`  | `start` | `nextResult` | `start` |

## Host

Maintain mappings & caches for host language classes & instances.

| Field Name     | Python | Ruby | Java | Node.js |
| -------------- | ------ | ---- | ---- | ------- |
| `ffi_polar`    | x      | x    | x    | x       |
| `classes`      | x      | x    | x    | x       |
| `constructors` | x      | x    | x    |         |
| `instances`    | x      | x    | x    | x       |

| Method              | Python | Ruby | Java | Node.js |
| ------------------- | ------ | ---- | ---- | ------- |
| delete              | x      | x    | x    |         |
| copy                | x      | x    | x    | x       |
| `get_class`         | x      | x    | x    | x       |
| `cache_class`       | x      | x    | x    | x       |
| `get_constructor`   | x      | x    |      |         |
| `get_instance`      | x      | x    | x    | x       |
| `has_instance`      |        | x    | x    | x       |
| `cache_instance`    | x      | x    | x    | x       |
| `make_instance`     | x      | x    | x    | x       |
| `isa`               | x      | x    | x    | x       |
| `is_subspecializer` | x      | x    | x    | x       |
| `unify`             | x      | x    |      | x       |
| `operator`          | x      |      |      |         |
| `to_polar_term`     | x      | x    | x    | x       |
| to_host             | x      | x    | x    | x       |

## Messages

### Handle Message Types

| MessageType | Python | Ruby | Java | Node.js |
| ----------- | ------ | ---- | ---- | ------- |
| Print       | x      | x    | x    | x       |
| Warning     | x      | x    | x    | x       |

### Check Messages After FFI Calls

| FFI call              | Python | Ruby | Java | Node.js |
| --------------------- | ------ | ---- | ---- | ------- |
| `load`                | x      | x    | x    | x       |
| `new_query_from_str`  | x      | x    | x    | x       |
| `new_query_from_term` | x      | x    | x    | x       |
| `next_inline_query`   | x      | x    | x    | x       |
| `next_query_event`    | x      | x    | x    | x       |
| `debug_command`       | x      | x    | x    | x       |
