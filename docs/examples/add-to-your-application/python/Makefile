.PHONY: test fmt

test: venv
	. venv/bin/activate; pytest app/test.py

fmt: venv
	. venv/bin/activate; black .

venv: venv/touchfile

venv/touchfile: requirements.txt
	test -d venv || python3 -m venv venv
	. venv/bin/activate; pip install -r requirements.txt
	touch $@
