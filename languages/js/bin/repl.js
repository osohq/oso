#!/usr/bin/env node

const { Oso } = require('../dist/src/Oso');

new Oso().repl(process.argv.slice(2));
