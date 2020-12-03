DIRS := oso django-oso sqlalchemy-oso flask-oso

lint: lint-requirements typecheck $(DIRS)

$(DIRS):
	cd $@ && flake8 .

typecheck:
	$(MAKE) -C oso typecheck
	$(MAKE) -C sqlalchemy-oso typecheck

lint-requirements: .make.lint-requirements.installed

.make.lint-requirements.installed: requirements-lint.txt
	pip install -r requirements-lint.txt
	touch $@

.PHONY: lint lint-requirements $(DIRS)
