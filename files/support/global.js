var _mapping = {};
var _pixelBuffer;
var state = {};
var start = new Date().getTime();

function _internalRegister() {
    return JSON.stringify(register());
}

function _getState() {
    return JSON.stringify(state);
}

function _setState(newstate) {
    state = JSON.parse(newstate);
}

function _setup(nm) {
    _mapping = JSON.parse(nm);

    _pixelBuffer = new Float64Array(_mapping.length * 3);
}

function _internalRender() {
    if (typeof beforeRender === "function") {
        let delta = new Date().getTime();
        beforeRender(start - delta);
        start = delta;
    }

    _mapping.forEach((m) => {
        render(m.I);
    });

    return _pixelBuffer;
}

function rgb(i, r, g, b) {
    _pixelBuffer[i * 3] = r;
    _pixelBuffer[i * 3 + 1] = g;
    _pixelBuffer[i * 3 + 2] = b;
}

function hsv(index, h, s, v) {
    var r, g, b, i, f, p, q, t;
    i = Math.floor(h * 6);
    f = h * 6 - i;
    p = v * (1 - s);
    q = v * (1 - f * s);
    t = v * (1 - (1 - f) * s);
    switch (i % 6) {
        case 0:
            (r = v), (g = t), (b = p);
            break;
        case 1:
            (r = q), (g = v), (b = p);
            break;
        case 2:
            (r = p), (g = v), (b = t);
            break;
        case 3:
            (r = p), (g = q), (b = v);
            break;
        case 4:
            (r = t), (g = p), (b = v);
            break;
        case 5:
            (r = v), (g = p), (b = q);
            break;
    }

    _pixelBuffer[index * 3] = r;
    _pixelBuffer[index * 3 + 1] = g;
    _pixelBuffer[index * 3 + 2] = b;
}

/// Easy shorthands

let max = Math.max;
let floor = Math.floor;
