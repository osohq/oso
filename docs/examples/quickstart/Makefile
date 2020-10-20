.PHONY: test test-ci

test:
	$(MAKE) -C ../../.. update-submodules
	$(MAKE) test-ci

test-ci:
	ruby test_quickstarts.rb
