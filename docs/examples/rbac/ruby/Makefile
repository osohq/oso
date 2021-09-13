.PHONY: test install

test: install
	bundle exec rspec .

install: .make.dependencies.installed

.make.dependencies.installed: Gemfile
	bundle
	@touch $@

