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

    _pixelBuffer = new Float64Array(_mapping.length * 4);
}

function _internalRender(frame) {
    if (typeof beforeRender === "function") {
        let delta = new Date().getTime();
        beforeRender(frame, start - delta);
        start = delta;
    }

    if (typeof render3D === "function") {
        _mapping.forEach((m) => {
            render3D(m.I, m.O[0], m.O[1], m.O[2]);
        });
    } else {
        _mapping.forEach((m) => {
            render(m.I);
        });
    }

    return _pixelBuffer;
}

function rgb(index, r, g, b) {
    _pixelBuffer[index * 4] = r;
    _pixelBuffer[index * 4 + 1] = g;
    _pixelBuffer[index * 4 + 2] = b;
    _pixelBuffer[index * 4 + 3] = 1.0;
}

function rgba(index, r, g, b, a) {
    _pixelBuffer[index * 4] = r;
    _pixelBuffer[index * 4 + 1] = g;
    _pixelBuffer[index * 4 + 2] = b;
    _pixelBuffer[index * 4 + 3] = a;
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

    _pixelBuffer[index * 4] = r;
    _pixelBuffer[index * 4 + 1] = g;
    _pixelBuffer[index * 4 + 2] = b;
    _pixelBuffer[index * 4 + 3] = 1.0;
}

/// Frame Functions
function sin(phase, cycle) {
    return Math.sin(phase * Math.PI * cycle);
}

function cos(phase, cycle) {
    return Math.cos(phase * Math.PI * cycle);
}

function triangle(phase) {
    return Math.acos(Math.sin(phase)) / 1.570796326794896;
}

/// Easy shorthands

let max = Math.max;
let floor = Math.floor;
let PI = Math.PI;
let PI2 = Math.PI2;

//Other (PixelBlaze compat)

function random(max) {
    return Math.random() * max;
}

function abs(number) {
    return Math.abs(number);
}
