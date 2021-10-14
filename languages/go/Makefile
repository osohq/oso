# NOTE: This only works on linux and osx
# (but then again none of our makefiles work on windows so it's not surprising)
# To see how we build all the release versions of the lib and include them see the
# release github action.
# Only x64 for now.

UNAME_S := $(shell uname -s)

copy_lib:
	cp ../../polar-c-api/polar.h internal/ffi/native/
ifeq ($(UNAME_S),Linux)
	cp ../../target/debug/libpolar.a internal/ffi/native/linux/
endif
ifeq ($(UNAME_S),Darwin)
	cp ../../target/debug/libpolar.a internal/ffi/native/macos/
endif

test: copy_lib
	go test -v ./tests/ -count=1

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
