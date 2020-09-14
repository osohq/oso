.PHONY: rust test package

rust:
	$(MAKE) -C ../.. rust-build
	# @NOTE: this is just the debug lib. Dont use this for release builds.
	if [ -e "../../target/debug/libpolar.dylib" ]; then \
		mkdir -p oso/src/main/resources/macos; \
		cp ../../target/debug/libpolar.dylib oso/src/main/resources/macos/; \
	fi
	if [ -e "../../target/debug/libpolar.so" ]; then \
		mkdir -p oso/src/main/resources/linux; \
		cp ../../target/debug/libpolar.so oso/src/main/resources/linux/; \
	fi

test: rust
	cd oso && mvn $(MVN_FLAGS) clean test

package: rust
	cd oso && mvn $(MVN_FLAGS) clean package

repl: rust
	cd oso && mvn $(MVN_FLAGS) exec:java -Dexec.mainClass="com.osohq.oso.Oso" -Dexec.args="$(ARGS)"

fmt:
	$(MAKE) -C ../.. java-fmt
