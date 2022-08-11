#!/usr/bin/env node

const { Oso } = require('../dist/src/Oso'); // eslint-disable-line node/no-missing-require

new Oso().repl(process.argv.slice(2));
