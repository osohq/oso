test-local:
	$(MAKE) go-local
	ruby test_quickstarts.rb

test-release:
	ruby test_quickstarts.rb

go-local:
	cd go && go mod edit -replace github.com/osohq/go-oso@v0.20.1=../../../../languages/go
