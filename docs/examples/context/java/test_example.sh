#!/usr/bin/env bash

# This is just a WOW HACK to get these tests running before there's a real oso java package I can reference.
javac -d ./build -classpath "../../../../languages/java/polar/lib/*:build:." ../../../../languages/java/polar/src/*.java ./*java
ENV="development" POLAR_LIB_PATH="../../../../languages/java/polar/lib/libpolar.dylib" java -classpath "../../../../languages/java/polar/lib/*:build" TestExample