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

    return _pixelBuffer;
}

function rgb(i, r, g, b) {
    _pixelBuffer[i * 3] = r;
    _pixelBuffer[i * 3 + 1] = g;
    _pixelBuffer[i * 3 + 2] = b;
}

function hsv(i, r, g, b) {}
