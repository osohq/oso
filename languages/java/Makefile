.PHONY: rust test package

rust:
	$(MAKE) -C ../.. rust-build

test: rust
	cd oso && mvn clean test

package: rust
	cd oso && mvn clean package
