.PHONY: test rust-test rust-build python-build python-test python-flask-build \
	python-flask-test python-django-test python-sqlalchemy-test ruby-test \
	java-test docs-test fmt clippy lint wasm-build wasm-test js-test

test: rust-test python-test ruby-test java-test python-flask-test \
	python-django-test python-sqlalchemy-test wasm-test js-test

rust-test:
	cargo test --all-targets

rust-build:
	cargo build

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
	python examples/expenses-py/app.py
	cd test && python test.py

python-flask-test: python-build python-flask-build
	$(MAKE) -C languages/python/flask-oso test

python-django-test: python-build python-django-build
	$(MAKE) -C languages/python/django-oso test

python-sqlalchemy-test: python-build python-sqlalchemy-build
	$(MAKE) -C languages/python/sqlalchemy-oso test

ruby-test:
	$(MAKE) -C languages/ruby test

java-test:
	$(MAKE) -C languages/java package
	cd test && \
		javac -classpath "../languages/java/oso/target/*:." Test.java && \
		java -classpath "../languages/java/oso/target/*:." -enableassertions Test

# Ensure jq is installed.
$(if $(shell command -v jq 2> /dev/null),,$(error Please install jq <https://stedolan.github.io/jq/>))

fmt.jar:
	$(eval URL := $(shell curl -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/google/google-java-format/releases/latest | jq '.assets[] | select(.name | test("all-deps.jar")) | .browser_download_url'))
	curl -L $(URL) > fmt.jar

java-fmt: fmt.jar
	$(eval FILES := $(shell git ls-files '*.java'))
	java -jar fmt.jar --replace $(FILES)

docs-test: python-build
	$(MAKE) -C docs test

fmt: java-fmt
	cargo fmt
	$(MAKE) -C languages/python/oso fmt
	$(MAKE) -C languages/python/flask-oso fmt
	$(MAKE) -C languages/python/django-oso fmt
	$(MAKE) -C languages/python/sqlalchemy-oso fmt
	$(MAKE) -C languages/js fmt

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

lint: clippy
	$(MAKE) -C languages/ruby lint typecheck
	$(MAKE) -C languages/js lint
	$(MAKE) fmt

wasm-build:
	$(MAKE) -C polar-wasm-api build

wasm-test:
	$(MAKE) -C polar-wasm-api test

js-test:
	$(MAKE) -C languages/js parity
	$(MAKE) -C languages/js test
