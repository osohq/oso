const PREFIX = 'oso';

function get(k) {
    return localStorage.getItem(`${PREFIX}_${k}`);
}

function set(k, v) {
    localStorage.setItem(`${PREFIX}_${k}`, v);
}

export {get, set};
