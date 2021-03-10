.PHONY: test lint fmt

test:
	cargo test -p oso --all-targets
	cargo test -p oso-derive

lint: fmt

fmt:
	cd ../.. && cargo fmt
