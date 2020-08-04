import { Polar as FfiPolar } from './wasm/polar';

interface PolarFile {
  name: string;
  contents: string;
}

class Polar {
  #ffiPolar: FfiPolar;
  // #host: Host;
  #loadQueue: Set<PolarFile>;

  constructor() {
    this.#ffiPolar = new FfiPolar();
    this.#loadQueue = new Set();
  }
}
