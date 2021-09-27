run: api_docs install
	hugo server

dev: install
	HUGO_PURGECSS=false hugo server

export JAVA_DIR := $(abspath ../languages/java/oso)
export JS_DIR := $(abspath ../languages/js)
export PYTHON_DIR := $(abspath ../languages/python)
export RUBY_DIR := $(abspath ../languages/ruby)

rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

JAVA_FILES := $(call rwildcard ../languages/java/oso/src,*.java)
JS_FILES := $(call rwildcard ../languages/js/src,*.ts)
PYTHON_FILES := $(call rwildcard,../languages/python,*.py)
PYTHON_RST_FILES := $(call rwildcard,../languages/python,*.rst)

RUBY_FILES := $(call rwildcard,../languages/ruby/lib,*.rb)

require-%:
	$(if ${${*}},,$(error You must pass the $* environment variable))

content/java/reference/api: $(JAVA_FILES)
	cd "$(JAVA_DIR)" && mvn -q javadoc:javadoc
	rm -rf $@
	cp -R "$(JAVA_DIR)/target/site/apidocs" $@

content/node/reference/api: $(JS_FILES)
	$(MAKE) -C "$(JS_DIR)" docs
	rm -rf $@
	cp -R "$(JS_DIR)/docs" $@

content/python/reference/api: $(PYTHON_FILES) $(PYTHON_RST_FILES)
	$(MAKE) -C "$(PYTHON_DIR)" docs
	rm -rf $@
	cp -R "$(PYTHON_DIR)/docs/_build/html" $@

content/ruby/reference/api: $(RUBY_FILES)
	cd "$(RUBY_DIR)" && yard doc
	rm -rf $@
	cp -R "$(RUBY_DIR)/doc" $@

api_docs:content/java/reference/api content/node/reference/api content/python/reference/api content/ruby/reference/api

examples:
	$(MAKE) -C examples submodules

install: examples global-deps tailwind-deps webpack-deps
	# uncomment when we want to include Oso WASM
	# $(MAKE) -C .. wasm-build
	# $(MAKE) -C ../languages/js build

global-deps: .make.global-deps.installed

.make.global-deps.installed:
	npm install -g postcss postcss-cli autoprefixer
	touch $@

tailwind-deps: .make.tailwind-deps.installed

.make.tailwind-deps.installed: themes/oso-tailwind/package.json
	cd themes/oso-tailwind && npm install
	touch $@

webpack-deps: .make.webpack-deps.installed

.make.webpack-deps.installed: themes/oso-webpack/package.json
	cd themes/oso-webpack && npm install && npm run-script build
	touch $@

build: install api_docs clean
	HUGO_ENV=production hugo -b https://docs-preview.oso.dev/

build-prod: install api_docs
	cd themes/oso-webpack && npm run-script build-prod
	HUGO_ENV=production hugo -b https://docs.osohq.com/

# Look for the s3deploy binary: https://github.com/bep/s3deploy
S3DEPLOY := $(shell command -v s3deploy 2> /dev/null)

deploy-ci: install api_docs clean
ifndef S3DEPLOY
	$(error "you need to install s3deploy to deploy the docs")
endif
ifndef DOCS_TAG
	$(error "you must set the DOCS_TAG environment variable")
endif
	cd themes/oso-webpack && npm run-script build-prod
	HUGO_ENV=production hugo -b https://docs-preview.oso.dev/v/$(DOCS_TAG)/
	s3deploy -bucket docs-preview.oso.dev -distribution-id E2KU2V8C9KJNU7 -path v/$(DOCS_TAG) -source public -region us-east-1

test: install
	### TODO: Add Hugo tests
	$(MAKE) -C examples test

clean:
	ls -A public | grep -v 'favicon.ico' | xargs -I {} echo 'public/'{} | xargs rm -rf

build-search:
	cd search && go mod vendor && go build -o ../searcher

require-algolia-vars: require-ALGOLIA_APPLICATION_ID require-ALGOLIA_ADMIN_API_KEY require-ALGOLIA_INDEX

## This is the entry for all doc stuff for preview
ci-preview-docs: require-ALGOLIA_ADMIN_API_KEY build build-search
	./searcher -index='preview_OSODOCS' -app_id='KROZ8F05YT'

## This is the entry for all doc stuff for production
ci-prod-docs: require-ALGOLIA_ADMIN_API_KEY build-prod build-search
	./searcher -index='prod_OSODOCS' -app_id='KROZ8F05YT'

search: require-algolia-vars build-prod build-search
	./searcher

spelling:
	$(MAKE) -C spelling

check-links: check-links-deps
	lychee --verbose './content/**/*.md' --exclude '\w+@\w+\.com' --exclude 'localhost|127.0.0.1' --exclude '^sqlite://' --exclude 'https://crates.io/crates'

check-links-deps: .make.check-links-deps.installed

.make.check-links-deps.installed:
	cargo install lychee
	touch $@

lint: build spelling check-links

test-quickstarts-release:
	$(MAKE) -C examples test-quickstarts-release

test-quickstarts-local:
	$(MAKE) -C examples test-quickstarts-local

.PHONY: api_docs run build examples global-deps tailwind-deps webpack-deps clean spelling build-search search require-algolia-vars check-links check-links-deps lint test-quickstarts-release test-quickstarts-local
