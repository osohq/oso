let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(String.raw`util`);

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error('expected a number argument');
}

let WASM_VECTOR_LEN = 0;

let cachegetNodeBufferMemory0 = null;
function getNodeBufferMemory0() {
    if (cachegetNodeBufferMemory0 === null || cachegetNodeBufferMemory0.buffer !== wasm.memory.buffer) {
        cachegetNodeBufferMemory0 = Buffer.from(wasm.memory.buffer);
    }
    return cachegetNodeBufferMemory0;
}

function passStringToWasm0(arg, malloc) {

    if (typeof(arg) !== 'string') throw new Error('expected a string argument');

    const len = Buffer.byteLength(arg);
    const ptr = malloc(len);
    getNodeBufferMemory0().write(arg, ptr, len);
    WASM_VECTOR_LEN = len;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

const u32CvtShim = new Uint32Array(2);

const uint64CvtShim = new BigUint64Array(u32CvtShim.buffer);

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error('expected a boolean argument');
    }
}

function logError(f) {
    return function () {
        try {
            return f.apply(this, arguments);

        } catch (e) {
            let error = (function () {
                try {
                    return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
                } catch(_) {
                    return "<failed to stringify thrown value>";
                }
            }());
            console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
            throw e;
        }
    };
}
/**
*/
class Polar {

    static __wrap(ptr) {
        const obj = Object.create(Polar.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_polar_free(ptr);
    }
    /**
    */
    constructor() {
        var ret = wasm.polar_wasm_new();
        return Polar.__wrap(ret);
    }
    /**
    * @param {string} src
    * @param {string | undefined} filename
    */
    loadFile(src, filename) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        wasm.polar_loadFile(this.ptr, ptr0, len0, ptr1, len1);
    }
    /**
    * @param {string} name
    * @param {Term} value
    */
    registerConstant(name, value) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        _assertClass(value, Term);
        if (value.ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr1 = value.ptr;
        value.ptr = 0;
        wasm.polar_registerConstant(this.ptr, ptr0, len0, ptr1);
    }
    /**
    * @returns {Query | undefined}
    */
    nextInlineQuery() {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ret = wasm.polar_nextInlineQuery(this.ptr);
        return ret === 0 ? undefined : Query.__wrap(ret);
    }
    /**
    * @param {string} src
    * @returns {Query}
    */
    newQueryFromStr(src) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.polar_newQueryFromStr(this.ptr, ptr0, len0);
        return Query.__wrap(ret);
    }
    /**
    * @param {Term} term
    * @returns {Query}
    */
    newQueryFromTerm(term) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        _assertClass(term, Term);
        if (term.ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr0 = term.ptr;
        term.ptr = 0;
        var ret = wasm.polar_newQueryFromTerm(this.ptr, ptr0);
        return Query.__wrap(ret);
    }
    /**
    * @returns {BigInt}
    */
    newId() {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        wasm.polar_newId(8, this.ptr);
        var r0 = getInt32Memory0()[8 / 4 + 0];
        var r1 = getInt32Memory0()[8 / 4 + 1];
        u32CvtShim[0] = r0;
        u32CvtShim[1] = r1;
        const n0 = uint64CvtShim[0];
        return n0;
    }
}
module.exports.Polar = Polar;
/**
*/
class Query {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    static __wrap(ptr) {
        const obj = Object.create(Query.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_query_free(ptr);
    }
    /**
    * @returns {any}
    */
    nextEvent() {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ret = wasm.query_nextEvent(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {BigInt} call_id
    * @param {Term | undefined} value
    */
    callResult(call_id, value) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        uint64CvtShim[0] = call_id;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        let ptr1 = 0;
        if (!isLikeNone(value)) {
            _assertClass(value, Term);
            if (value.ptr === 0) {
                throw new Error('Attempt to use a moved value');
            }
            ptr1 = value.ptr;
            value.ptr = 0;
        }
        wasm.query_callResult(this.ptr, low0, high0, ptr1);
    }
    /**
    * @param {BigInt} call_id
    * @param {boolean} result
    */
    questionResult(call_id, result) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        uint64CvtShim[0] = call_id;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        _assertBoolean(result);
        wasm.query_questionResult(this.ptr, low0, high0, result);
    }
    /**
    * @param {string} command
    */
    debugCommand(command) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.query_debugCommand(this.ptr, ptr0, len0);
    }
    /**
    * @param {string} msg
    */
    appError(msg) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passStringToWasm0(msg, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.query_appError(this.ptr, ptr0, len0);
    }
}
module.exports.Query = Query;
/**
* Represents a concrete instance of a Polar value
*/
class Term {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_term_free(ptr);
    }
}
module.exports.Term = Term;

module.exports.__wbindgen_number_new = function(arg0) {
    var ret = arg0;
    return addHeapObject(ret);
};

module.exports.__wbindgen_string_new = function(arg0, arg1) {
    var ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

module.exports.__wbindgen_object_clone_ref = function(arg0) {
    var ret = getObject(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_new_68adb0d58759a4ed = logError(function() {
    var ret = new Object();
    return addHeapObject(ret);
});

module.exports.__wbg_set_2e79e744454afade = logError(function(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
});

module.exports.__wbg_new_a938277eeb06668d = logError(function() {
    var ret = new Array();
    return addHeapObject(ret);
});

module.exports.__wbg_push_2bfc5fcfa4d4389d = logError(function(arg0, arg1) {
    var ret = getObject(arg0).push(getObject(arg1));
    _assertNum(ret);
    return ret;
});

module.exports.__wbg_new_03a50b2d1045a68f = logError(function(arg0, arg1) {
    var ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
});

module.exports.__wbg_setname_237826d8e1e796be = logError(function(arg0, arg1, arg2) {
    getObject(arg0).name = getStringFromWasm0(arg1, arg2);
});

module.exports.__wbg_new_bd7709fb051ded5c = logError(function() {
    var ret = new Map();
    return addHeapObject(ret);
});

module.exports.__wbg_set_c784587ca3fcfb04 = logError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
});

module.exports.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_rethrow = function(arg0) {
    throw takeObject(arg0);
};

const path = require('path').join(__dirname, 'polar_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

