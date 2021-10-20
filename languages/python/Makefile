docs: docs-requirements
	cd docs && sphinx-build -M html . _build

docs-requirements: .make.docs-requirements.installed

.make.docs-requirements.installed: requirements-docs.txt requirements-local.txt
	pip install -r requirements-docs.txt
# Skip installing local version of packages
# when building published docs
ifeq ($(origin PUBLISH_DOCS), undefined)
	pip install -r requirements-local.txt
endif
	touch $@

clean:
	find ./ -name ".make.*" -exec rm -rf {} \;

lint: lint-oso lint-django-oso lint-flask-oso lint-sqlalchemy-oso typecheck

lint-oso: lint-requirements
	$(MAKE) -C oso lint

lint-django-oso: lint-requirements
	$(MAKE) -C django-oso lint

lint-flask-oso: lint-requirements
	$(MAKE) -C flask-oso lint

lint-sqlalchemy-oso: lint-requirements
	$(MAKE) -C sqlalchemy-oso lint

fmt: fmt-oso fmt-django-oso fmt-flask-oso fmt-sqlalchemy-oso

fmt-oso: fmt-requirements
	$(MAKE) -C oso fmt

fmt-django-oso: fmt-requirements
	$(MAKE) -C django-oso fmt

fmt-flask-oso: fmt-requirements
	$(MAKE) -C flask-oso fmt

fmt-sqlalchemy-oso: fmt-requirements
	$(MAKE) -C sqlalchemy-oso fmt

typecheck: lint-requirements
	$(MAKE) -C oso typecheck
	$(MAKE) -C sqlalchemy-oso typecheck

lint-requirements: .make.lint-requirements.installed

.make.lint-requirements.installed: requirements-lint.txt
	pip install -r requirements-lint.txt
	touch $@

fmt-requirements: .make.fmt-requirements.installed

.make.fmt-requirements.installed: requirements-fmt.txt
	pip install -r requirements-fmt.txt
	touch $@

.PHONY: docs docs-requirements typecheck \
	fmt-requirements fmt fmt-oso fmt-django-oso fmt-flask-oso fmt-sqlalchemy-oso \
	lint-requirements lint lint-oso lint-django-oso lint-flask-oso lint-sqlalchemy-oso clean
