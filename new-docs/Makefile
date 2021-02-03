export RUBY_DIR := $(abspath ../languages/ruby)
export JAVA_DIR := $(abspath ../languages/java/oso)
export JS_DIR := $(abspath ../languages/js)

rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

RUBY_FILES := $(call rwildcard,../languages/ruby/lib,*.rb)
JAVA_FILES := $(call rwildcard ../languages/java/oso/src,*.java)
JS_FILES := $(call rwildcard ../languages/js/src,*.ts)

content/ruby/reference/api: $(RUBY_FILES)
	cd "$(RUBY_DIR)" && yard doc
	rm -rf $@
	cp -R "$(RUBY_DIR)/doc" $@

content/java/reference/api: $(JAVA_FILES)
	cd "$(JAVA_DIR)" && mvn -q javadoc:javadoc
	rm -rf $@
	cp -R "$(JAVA_DIR)/target/site/apidocs" $@

content/node/reference/api: $(JS_FILES)
	$(MAKE) -C "$(JS_DIR)" docs
	rm -rf $@
	cp -R "$(JS_DIR)/docs" $@

api_docs: content/ruby/reference/api content/java/reference/api content/node/reference/api

.PHONY: api_docs

install:
	$(MAKE) -C .. wasm-build
	$(MAKE) -C ../languages/js build
	cd themes/tailwind && npm install
	cd themes/oso && npm install && npm run-script build

run: api_docs
	hugo server

build: install
	HUGO_ENV=production hugo --minify
