oso = require('./dist/src/oso.js');
require('dotenv').config()
console.log(process.env)
class Foo{}

var o = new oso.Oso();
var isaCheck = (name) => (i) => i !== undefined && "typename" in i && i.typename == name;

o.registerClass(Foo, {name: "Foo", isaCheck: isaCheck("Foo") });
o.registerConstant({typename: "Foo", id: 1}, "foo")
o.repl()
