import { resolve } from 'path';

import * as Mocha from 'mocha'; // eslint-disable-line node/no-unpublished-import
import * as glob from 'glob'; // eslint-disable-line node/no-unpublished-import

type Callback = (error?: Error, failures?: number) => void;

export function run(cwd: string, cb: Callback): void {
  const mocha = new Mocha({ ui: 'tdd', color: true, timeout: 5000 });

  glob('**/**.test.js', { cwd }, (err, files) => {
    if (err) return cb(err);

    // Add files to the test suite
    files.forEach(f => mocha.addFile(resolve(cwd, f)));

    try {
      mocha.run(failures => cb(null, failures));
    } catch (err) {
      console.error(err);
      cb(err);
    }
  });
}
