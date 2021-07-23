spelling: spelling-requirements sort-allowed-words
	pyspelling

spelling-requirements: .make.spelling-requirements.installed

.make.spelling-requirements.installed: requirements-spelling.txt
	pip install -r requirements-spelling.txt
	touch $@

sort-allowed-words: allowed_words.txt
	LANG=C sort -uo allowed_words.txt allowed_words.txt

.PHONY: spelling spelling-requirements sort-allowed-words
