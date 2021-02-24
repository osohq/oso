spelling: spelling-requirements
	pyspelling

spelling-requirements: .make.spelling-requirements.installed

.make.spelling-requirements.installed: requirements-spelling.txt
	pip install -r requirements-spelling.txt
	touch $@

.PHONY: spelling spelling-requirements
