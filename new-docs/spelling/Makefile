spelling: rest go java node python ruby rust

spelling-requirements: .make.spelling-requirements.installed

.make.spelling-requirements.installed: requirements-spelling.txt
	pip install -r requirements-spelling.txt
	touch $@

rest: spelling-requirements
	pyspelling -c .pyspelling.REST.yml

go java node python ruby rust: spelling-requirements
	cat .pyspelling.LANG.yml | sed 's/LANG/$@/g' > .pyspelling.$@.yml
	pyspelling -c .pyspelling.$@.yml

.PHONY: spelling spelling-requirements rest go java node python ruby rust
