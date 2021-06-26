state = {
    hue: 0.0,
    saturation: 0.0,
    value: 0.0,
};

function beforeRender(delta) {}

function render(index) {
    hsv(index, state.hue, state.saturation, state.value);
}
