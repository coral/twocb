var _mapping = {};
var _pixelBuffer;

function _internalRegister() {
    return JSON.stringify(register());
}

function _setup(nm) {
    _mapping = JSON.parse(nm);
    _pixelBuffer = new Float64Array(_mapping.Length * 3);
}

function _internalRender() {
    if (typeof beforeRender === "function") {
        beforeRender();
    }

    _mapping.forEach((m) => {
        render(m);
    });
}

function rgb(i, r, g, b) {
    x[i * 3] = r;
    x[i * 3 + 1] = g;
    x[i * 3 + 2] = b;
}

function hsv(i, r, g, b) {}
