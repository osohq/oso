# NOTE: This only works on linux and osx
# (but then again none of our makefiles work on windows so it's not surprising)
# To see how we build all the release versions of the lib and include them see the
# release github action.
# Only x64 for now.

UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

copy_lib:
	cp ../../polar-c-api/polar.h internal/ffi/native/
ifeq ($(UNAME_S),Linux)
ifeq ($(UNAME_M),x86_64)
	cp ../../target/debug/libpolar.a internal/ffi/native/linux/amd64/
else
	cp ../../target/debug/libpolar.a internal/ffi/native/linux/arm64/
endif
endif
ifeq ($(UNAME_S),Darwin)
ifeq ($(UNAME_M),x86_64)
	cp ../../target/debug/libpolar.a internal/ffi/native/macos/amd64/
else
	cp ../../target/debug/libpolar.a internal/ffi/native/macos/arm64/
endif
endif

test: copy_lib
	POLAR_IGNORE_NO_ALLOW_WARNING=1 go test -v ./internal/host -count=1
	cd tests && POLAR_IGNORE_NO_ALLOW_WARNING=1 go test -v -count=1

fmt:
	go fmt ./...

lint:
	if [ -n "$$(gofmt -l .)" ]; then \
		gofmt -l . && \
		exit 1; \
	fi

clean:
	go clean

.PHONY: copy_lib test fmt lint clean
