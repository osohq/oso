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

install:
	# uncomment when we want to include Oso WASM
	# $(MAKE) -C .. wasm-build
	# $(MAKE) -C ../languages/js build
	npm install -g postcss postcss-cli autoprefixer
	cd themes/oso-tailwind && npm install
	cd themes/oso-webpack && npm install && npm run-script build

run: api_docs
	hugo server

build: install
	cd themes/oso-webpack && npm run-script build
	HUGO_ENV=production hugo -b https://docs-preview.oso.dev/

build-prod: install
	cd themes/oso-webpack && npm run-script build-prod
	HUGO_ENV=production hugo -b https://docs.osohq.com/

.PHONY: api_docs run build
