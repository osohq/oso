import path from 'path';

import { Oso } from './Oso';

const oso = new Oso();
oso.loadFile(path.resolve(__dirname, '../test.polar'));
console.log(oso.query('isTrue()').next());
