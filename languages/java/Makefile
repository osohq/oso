.PHONY: rust test package

rust:
	$(MAKE) -C ../.. rust-build

test: rust
	cd oso && mvn test

package: rust
	cd oso && mvn package
