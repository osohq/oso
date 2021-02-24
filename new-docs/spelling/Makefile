spelling: spelling-requirements
	pyspelling

spelling-requirements: .make.spelling-requirements.installed sort-allowed-words

.make.spelling-requirements.installed: requirements-spelling.txt
	pip install -r requirements-spelling.txt
	touch $@

sort-allowed-words:
	sort -fo allowed_words.txt allowed_words.txt

.PHONY: spelling spelling-requirements sort-allowed-words
