Parity tests
============

These tests ensure parity with the old Python Polar implementation. They
are run in CI against both versions and ensure everything passes.

New tests that are integration style and are meant to test behavior that
should be the same in both codebases should be written here.

To ensure these tests can be run against both codebases, they *must*:

- only use api.py or exceptions.py
- only use test_helpers.py for external fixtures.
