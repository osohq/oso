check: deps
	bundle exec htmlproofer ../public --empty-alt-ignore --allow-hash-href --file-ignore '/public/\w+/reference/api/' --url-ignore '/.linkedin.com/,/.hs-scripts.com/,/docs-preview.oso.dev/\w+\/reference\/api\//'

deps: .make.deps.installed

.make.deps.installed: Gemfile
	bundle
	touch $@

.PHONY: check deps
