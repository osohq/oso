import fs from 'fs/promises';

import { Polar } from './polar';

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
        event.Result.bindings.forEach((v, k) => {
          console.log(`${k} => ${this.toJs(v)}`);
        });
      }
    }
  }

  private toJs(v: PolarValue): any {
    const t = v.value;
    if (isPolarStr(t)) {
      return t.String;
    } else if (isPolarNum(t)) {
      if ('Float' in t.Number) {
        return t.Number.Float;
      } else {
        return t.Number.Integer;
      }
    } else if (isPolarBool(t)) {
      return t.Boolean;
    } else if (isPolarList(t)) {
      return t.List.map(this.toJs);
    } else if (isPolarDict(t)) {
      const copy = {};
      Object.entries(t.Dictionary.fields).forEach(
        ([k, v]) => (copy[k] = this.toJs(v))
      );
      return copy;
    }
  }
}

interface PolarStr {
  String: string;
}

function isPolarStr(v: PolarType): v is PolarStr {
  return (v as PolarStr).String !== undefined;
}

interface PolarNum {
  Number: PolarFloat | PolarInt;
}

function isPolarNum(v: PolarType): v is PolarNum {
  return (v as PolarNum).Number !== undefined;
}

interface PolarFloat {
  Float: number;
}

interface PolarInt {
  Integer: number;
}

interface PolarBool {
  Boolean: boolean;
}

function isPolarBool(v: PolarType): v is PolarBool {
  return (v as PolarBool).Boolean !== undefined;
}

interface PolarList {
  List: PolarValue[];
}

function isPolarList(v: PolarType): v is PolarList {
  return (v as PolarList).List !== undefined;
}

interface PolarDict {
  Dictionary: {
    fields: {
      [key: string]: PolarValue;
    };
  };
}

function isPolarDict(v: PolarType): v is PolarDict {
  return (v as PolarDict).Dictionary !== undefined;
}

type PolarType = PolarStr | PolarNum | PolarBool | PolarList | PolarDict;

interface PolarValue {
  value: PolarType;
}

async function main() {
  const oso = new Oso();
  await oso.loadFile('dist/test.polar');
  oso.queryStr('f(x)');
}

main();
