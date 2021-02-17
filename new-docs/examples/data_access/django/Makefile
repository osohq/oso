test: setup
	pytest example

setup:
	pip install -r requirements.txt
	python example/manage.py makemigrations app
	python example/manage.py migrate
	python example/manage.py seed

.PHONY: test setup
