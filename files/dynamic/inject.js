state = {
    hue: 0.0,
    saturation: 0.0,
    value: 0.0,
};

function beforeRender(frame, delta) {}

function render(index) {
    hsv(index, state.hue, 1.0, 1.0);
}
