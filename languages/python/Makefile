lint: lint-requirements typecheck
	$(MAKE) -C oso lint
	$(MAKE) -C django-oso lint
	$(MAKE) -C sqlalchemy-oso lint
	$(MAKE) -C flask-oso lint

typecheck:
	$(MAKE) -C oso typecheck
	$(MAKE) -C sqlalchemy-oso typecheck

lint-requirements: .make.lint-requirements.installed

.make.lint-requirements.installed: requirements-lint.txt
	pip install -r requirements-lint.txt
	touch $@

.PHONY: lint lint-requirements typecheck
