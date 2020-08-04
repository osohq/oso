import fs from 'fs/promises';

import { Polar } from './wasm/polar';

interface PolarFile {
  name: string;
  contents: string;
}

class Oso {
  #polar: Polar;
  #loadQueue: Set<PolarFile>;
  constructor() {
    this.#polar = new Polar();
    this.#loadQueue = new Set();
  }

  async loadFile(name: string) {
    const parts = name.split('.');
    const extension = parts[parts.length - 1];
    if (parts.length <= 1 || !['pol', 'polar'].includes(extension)) {
      const e = new Error();
      e.name = 'PolarFileExtensionError';
      throw e;
    }
    try {
      const contents = await fs.readFile(name, 'utf8');
      this.#loadQueue.add({ name, contents });
    } catch (e) {
      if (e.code === 'ENOENT') {
        const e = new Error();
        e.name = 'PolarFileNotFoundError';
        throw e;
      } else {
        throw e;
      }
    }
  }

  private loadQueuedFiles() {
    try {
      this.#loadQueue.forEach((file) => {
        this.#polar.loadFile(file.contents, file.name);
        this.#loadQueue.delete(file);
      });
    } catch (e) {
      throw e;
    }
  }

  queryStr(str: string) {
    this.loadQueuedFiles();
    const query = this.#polar.newQueryFromStr(str);
    while (true) {
      const event = query.nextEvent();
      if (event === 'Done') break;
      if (event.Result) {
        // event.Result.bindings.forEach((v, k) => {
        //   console.log(`${k} => ${this.toJs(v)}`);
        // });
      }
    }
  }
}

async function main() {
  const oso = new Oso();
  await oso.loadFile('dist/test.polar');
  oso.queryStr('f(x)');
}

main();
