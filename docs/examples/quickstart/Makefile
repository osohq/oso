.PHONY: test-local test-release

test-local:
	$(MAKE) -C ../../../languages/go copy_lib
	ruby test_quickstarts.rb local

test-release:
	ruby test_quickstarts.rb
