.PHONY: test go-test rust-test rust-build python-build python-test python-flask-build \
	python-flask-test python-django-test python-sqlalchemy-test ruby-test \
	java-test docs-test fmt clippy lint wasm-build wasm-test js-test \
	lint-ruby lint-js lint-go lint-java lint-rust fmt-java fmt-rust fmt-go fmt-js fmt-python \
	clean clean-docs clean-rust clean-go clean-js clean-polar-wasm-api

#! If you add another dependency to this you must also add it to the Test
#! github action or it won't run in CI. All jobs run in parallel on CI and
#! `make test` is just a local convenience.
test: go-test rust-test python-test ruby-test java-test python-flask-test \
	python-django-test python-sqlalchemy-test wasm-test js-test

rust-test:
	cargo test --all-targets --all-features

rust-build:
	cargo build -p polar-c-api

python-build: rust-build
	$(MAKE) -C languages/python/oso build

python-flask-build: python-build
	$(MAKE) -C languages/python/flask-oso build

python-django-build: python-build
	$(MAKE) -C languages/python/django-oso build

python-sqlalchemy-build: python-build
	$(MAKE) -C languages/python/sqlalchemy-oso build

python-test: python-build
	$(MAKE) -C languages/python/oso test
	cd test && python test.py

python-flask-test: python-build python-flask-build
	$(MAKE) -C languages/python/flask-oso test

python-django-test: python-build python-django-build
	$(MAKE) -C languages/python/django-oso test
	$(MAKE) -C languages/python/django-oso test22

python-sqlalchemy-test: python-build
	$(MAKE) -C languages/python/sqlalchemy-oso test

ruby-test:
	$(MAKE) -C languages/ruby test

OSO_VERSION :=$(shell cat VERSION)
java-test:
	$(MAKE) -C languages/java package
	cd test && \
		javac -classpath "../languages/java/oso/target/oso-$(OSO_VERSION).jar:." Test.java && \
		java -classpath "../languages/java/oso/target/oso-$(OSO_VERSION).jar:." -enableassertions Test

go-test: rust-build
	$(MAKE) -C languages/go test

docs-test: python-build
	$(MAKE) -C docs test

fmt: fmt-java fmt-rust fmt-python fmt-js fmt-go

# Ensure jq is installed.
$(if $(shell command -v jq 2> /dev/null),,$(error Please install jq <https://stedolan.github.io/jq/>))

fmt.jar:
	$(eval URL := $(shell curl -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/google/google-java-format/releases/latest | jq '.assets[] | select(.name | test("all-deps.jar")) | .browser_download_url'))
	curl -L $(URL) > fmt.jar

fmt-java: fmt.jar
	$(eval FILES := $(shell git ls-files 'languages/java/*.java'))
	$(eval OPENS := $(shell echo "--add-opens jdk.compiler/com.sun.tools.javac."{api,tree,file,util,parser}"=ALL-UNNAMED"))
	java $(OPENS) -jar fmt.jar --replace $(FILES)

fmt-rust:
	cargo fmt

fmt-go:
	$(MAKE) -C languages/go fmt

fmt-js:
	$(MAKE) -C languages/js fmt

fmt-python:
	$(MAKE) -C languages/python fmt

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

lint-python: python-build python-flask-build python-sqlalchemy-build python-django-build
	$(MAKE) -C languages/python lint

lint-ruby:
	$(MAKE) -C languages/ruby lint typecheck

lint-js:
	$(MAKE) -C languages/js lint

lint-go:
	$(MAKE) -C languages/go lint

lint-java:
	$(MAKE) -C languages/java lint

lint-rust:
	$(MAKE) -C languages/rust lint

lint: clippy lint-python lint-ruby lint-js lint-go lint-java lint-rust
	$(MAKE) fmt

wasm-build:
	$(MAKE) -C polar-wasm-api build

wasm-test:
	$(MAKE) -C polar-wasm-api test

js-test:
	$(MAKE) -C languages/js parity
	$(MAKE) -C languages/js test

clean: clean-docs clean-rust clean-go clean-java clean-js clean-polar-wasm-api clean-python

clean-docs:
	$(MAKE) -C docs clean

clean-rust:
	cargo clean

clean-go:
	$(MAKE) -C languages/go clean

clean-java:
	$(MAKE) -C languages/java clean

clean-js:
	$(MAKE) -C languages/js clean

clean-polar-wasm-api:
	$(MAKE) -C polar-wasm-api clean

clean-python:
	$(MAKE) -C languages/python clean
