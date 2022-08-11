let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextEncoder, TextDecoder } = require(`util`);

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let WASM_VECTOR_LEN = 0;

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

let heap_next = heap.length;

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

let cachegetFloat64Memory0 = null;
function getFloat64Memory0() {
    if (cachegetFloat64Memory0 === null || cachegetFloat64Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachegetFloat64Memory0;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
/**
*/
class Polar {

    static __wrap(ptr) {
        const obj = Object.create(Polar.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_polar_free(ptr);
    }
    /**
    */
    constructor() {
        var ret = wasm.polar_wasm_new();
        return Polar.__wrap(ret);
    }
    /**
    * @param {any} sources
    */
    load(sources) {
        wasm.polar_load(this.ptr, addHeapObject(sources));
    }
    /**
    */
    clearRules() {
        wasm.polar_clearRules(this.ptr);
    }
    /**
    * @param {string} name
    * @param {any} term
    */
    registerConstant(name, term) {
        var ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.polar_registerConstant(this.ptr, ptr0, len0, addHeapObject(term));
    }
    /**
    * @returns {Query | undefined}
    */
    nextInlineQuery() {
        var ret = wasm.polar_nextInlineQuery(this.ptr);
        return ret === 0 ? undefined : Query.__wrap(ret);
    }
    /**
    * @param {string} src
    * @returns {Query}
    */
    newQueryFromStr(src) {
        var ptr0 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.polar_newQueryFromStr(this.ptr, ptr0, len0);
        return Query.__wrap(ret);
    }
    /**
    * @param {any} term
    * @returns {Query}
    */
    newQueryFromTerm(term) {
        var ret = wasm.polar_newQueryFromTerm(this.ptr, addHeapObject(term));
        return Query.__wrap(ret);
    }
    /**
    * @returns {number}
    */
    newId() {
        var ret = wasm.polar_newId(this.ptr);
        return ret;
    }
    /**
    * @returns {any}
    */
    nextMessage() {
        var ret = wasm.polar_nextMessage(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {string} name
    * @param {any} mro
    */
    registerMro(name, mro) {
        var ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.polar_registerMro(this.ptr, ptr0, len0, addHeapObject(mro));
    }
    /**
    * @param {any} types
    * @param {any} partial_results
    * @param {string} variable
    * @param {string} class_tag
    * @returns {any}
    */
    buildDataFilter(types, partial_results, variable, class_tag) {
        var ptr0 = passStringToWasm0(variable, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ptr1 = passStringToWasm0(class_tag, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        var ret = wasm.polar_buildDataFilter(this.ptr, addHeapObject(types), addHeapObject(partial_results), ptr0, len0, ptr1, len1);
        return takeObject(ret);
    }
    /**
    * @param {boolean} ignore_no_allow_warning
    */
    setIgnoreNoAllowWarning(ignore_no_allow_warning) {
        wasm.polar_setIgnoreNoAllowWarning(this.ptr, ignore_no_allow_warning);
    }
}
module.exports.Polar = Polar;
/**
*/
class Query {

    static __wrap(ptr) {
        const obj = Object.create(Query.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_query_free(ptr);
    }
    /**
    * @returns {any}
    */
    nextEvent() {
        var ret = wasm.query_nextEvent(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {number} call_id
    * @param {any} term
    */
    callResult(call_id, term) {
        wasm.query_callResult(this.ptr, call_id, addHeapObject(term));
    }
    /**
    * @param {number} call_id
    * @param {boolean} result
    */
    questionResult(call_id, result) {
        wasm.query_questionResult(this.ptr, call_id, result);
    }
    /**
    * @param {string} command
    */
    debugCommand(command) {
        var ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.query_debugCommand(this.ptr, ptr0, len0);
    }
    /**
    * @param {string} msg
    */
    appError(msg) {
        var ptr0 = passStringToWasm0(msg, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.query_appError(this.ptr, ptr0, len0);
    }
    /**
    * @returns {any}
    */
    nextMessage() {
        var ret = wasm.query_nextMessage(this.ptr);
        return takeObject(ret);
    }
    /**
    * @returns {string}
    */
    source() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.query_source(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @param {string} name
    * @param {any} term
    */
    bind(name, term) {
        var ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.query_bind(this.ptr, ptr0, len0, addHeapObject(term));
    }
    /**
    * @param {string | undefined} rust_log
    * @param {string | undefined} polar_log
    */
    setLoggingOptions(rust_log, polar_log) {
        var ptr0 = isLikeNone(rust_log) ? 0 : passStringToWasm0(rust_log, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(polar_log) ? 0 : passStringToWasm0(polar_log, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        wasm.query_setLoggingOptions(this.ptr, ptr0, len0, ptr1, len1);
    }
}
module.exports.Query = Query;

module.exports.__wbindgen_string_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    var ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

module.exports.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

module.exports.__wbindgen_boolean_get = function(arg0) {
    const v = getObject(arg0);
    var ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};

module.exports.__wbindgen_number_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    var ret = typeof(obj) === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};

module.exports.__wbindgen_is_object = function(arg0) {
    const val = getObject(arg0);
    var ret = typeof(val) === 'object' && val !== null;
    return ret;
};

module.exports.__wbindgen_is_string = function(arg0) {
    var ret = typeof(getObject(arg0)) === 'string';
    return ret;
};

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

module.exports.__wbindgen_is_null = function(arg0) {
    var ret = getObject(arg0) === null;
    return ret;
};

module.exports.__wbindgen_is_undefined = function(arg0) {
    var ret = getObject(arg0) === undefined;
    return ret;
};

module.exports.__wbg_String_c8baaa0740def8c6 = function(arg0, arg1) {
    var ret = String(getObject(arg1));
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

module.exports.__wbg_get_2d1407dba3452350 = function(arg0, arg1) {
    var ret = getObject(arg0)[takeObject(arg1)];
    return addHeapObject(ret);
};

module.exports.__wbg_set_f1a4ac8f3a605b11 = function(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
};

module.exports.__wbg_new_693216e109162396 = function() {
    var ret = new Error();
    return addHeapObject(ret);
};

module.exports.__wbg_stack_0ddaca5d1abfb52f = function(arg0, arg1) {
    var ret = getObject(arg1).stack;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

module.exports.__wbg_error_09919627ac0992f5 = function(arg0, arg1) {
    try {
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(arg0, arg1);
    }
};

module.exports.__wbg_error_e793d7cbf03cad25 = function(arg0, arg1) {
    console.error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbg_get_67189fe0b323d288 = function(arg0, arg1) {
    var ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};

module.exports.__wbg_length_42e02f5a04d67464 = function(arg0) {
    var ret = getObject(arg0).length;
    return ret;
};

module.exports.__wbg_new_949bbc1147195c4e = function() {
    var ret = new Array();
    return addHeapObject(ret);
};

module.exports.__wbindgen_is_function = function(arg0) {
    var ret = typeof(getObject(arg0)) === 'function';
    return ret;
};

module.exports.__wbg_new_ac32179a660db4bb = function() {
    var ret = new Map();
    return addHeapObject(ret);
};

module.exports.__wbg_next_c4151d46d5fa7097 = function(arg0) {
    var ret = getObject(arg0).next;
    return addHeapObject(ret);
};

module.exports.__wbg_next_7720502039b96d00 = function() { return handleError(function (arg0) {
    var ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments) };

module.exports.__wbg_done_b06cf0578e89ff68 = function(arg0) {
    var ret = getObject(arg0).done;
    return ret;
};

module.exports.__wbg_value_e74a542443d92451 = function(arg0) {
    var ret = getObject(arg0).value;
    return addHeapObject(ret);
};

module.exports.__wbg_iterator_4fc4ce93e6b92958 = function() {
    var ret = Symbol.iterator;
    return addHeapObject(ret);
};

module.exports.__wbg_get_4d0f21c2f823742e = function() { return handleError(function (arg0, arg1) {
    var ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

module.exports.__wbg_call_888d259a5fefc347 = function() { return handleError(function (arg0, arg1) {
    var ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

module.exports.__wbg_new_0b83d3df67ecb33e = function() {
    var ret = new Object();
    return addHeapObject(ret);
};

module.exports.__wbg_isArray_eb7ad55f2da67dde = function(arg0) {
    var ret = Array.isArray(getObject(arg0));
    return ret;
};

module.exports.__wbg_push_284486ca27c6aa8b = function(arg0, arg1) {
    var ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

module.exports.__wbg_instanceof_ArrayBuffer_764b6d4119231cb3 = function(arg0) {
    var ret = getObject(arg0) instanceof ArrayBuffer;
    return ret;
};

module.exports.__wbg_values_364ae56c608e6824 = function(arg0) {
    var ret = getObject(arg0).values();
    return addHeapObject(ret);
};

module.exports.__wbg_new_342a24ca698edd87 = function(arg0, arg1) {
    var ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

module.exports.__wbg_setname_15d4109043e260cc = function(arg0, arg1, arg2) {
    getObject(arg0).name = getStringFromWasm0(arg1, arg2);
};

module.exports.__wbg_set_a46091b120cc63e9 = function(arg0, arg1, arg2) {
    var ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

module.exports.__wbg_isSafeInteger_0dfc6d38b7184f06 = function(arg0) {
    var ret = Number.isSafeInteger(getObject(arg0));
    return ret;
};

module.exports.__wbg_now_af172eabe2e041ad = function() {
    var ret = Date.now();
    return ret;
};

module.exports.__wbg_entries_aadf9c3f38203a12 = function(arg0) {
    var ret = Object.entries(getObject(arg0));
    return addHeapObject(ret);
};

module.exports.__wbg_buffer_397eaa4d72ee94dd = function(arg0) {
    var ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

module.exports.__wbg_new_a7ce447f15ff496f = function(arg0) {
    var ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

module.exports.__wbg_set_969ad0a60e51d320 = function(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

module.exports.__wbg_length_1eb8fc608a0d4cdb = function(arg0) {
    var ret = getObject(arg0).length;
    return ret;
};

module.exports.__wbg_instanceof_Uint8Array_08a1f3a179095e76 = function(arg0) {
    var ret = getObject(arg0) instanceof Uint8Array;
    return ret;
};

module.exports.__wbg_has_1275b5eec3dc7a7a = function() { return handleError(function (arg0, arg1) {
    var ret = Reflect.has(getObject(arg0), getObject(arg1));
    return ret;
}, arguments) };

module.exports.__wbindgen_debug_string = function(arg0, arg1) {
    var ret = debugString(getObject(arg1));
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_rethrow = function(arg0) {
    throw takeObject(arg0);
};

module.exports.__wbindgen_memory = function() {
    var ret = wasm.memory;
    return addHeapObject(ret);
};

const path = require('path').join(__dirname, 'polar_wasm_api_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

