DIRS := oso django-oso sqlalchemy-oso flask-oso

lint: lint-requirements typecheck $(DIRS)

$(DIRS):
	cd $@ && flake8 .

typecheck:
	cd oso && mypy .

lint-requirements:
	pip install -r requirements-lint.txt

.PHONY: lint $(DIRS)