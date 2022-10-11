#!/usr/bin/env node

const { Oso } = require('../dist/src/Oso'); // eslint-disable-line node/no-missing-require
const { Dict } = require('../dist/src/types');

const oso = new Oso();

class Foo { }

oso.registerClass(Foo, {
    isaCheck: (instance) => instance instanceof Object && instance.typeName && instance.typeName == "Foo"
});
oso.registerConstant({ typeName: "Foo" }, "foo");
oso.registerConstant(new Dict({ x: 1 }), "d")
oso.registerConstant({ x: 1 }, "d2")

oso.repl(process.argv.slice(2));
