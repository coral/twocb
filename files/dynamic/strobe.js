state = {
    hue: 1.0,
    saturation: 0.0,
};

var bar = 0.0;

function beforeRender(frame, delta) {
    if (frame.bar > 0.9) {
        bar = 1.0;
    } else {
        bar = 0.0;
    }
}

function render(index) {
    hsv(index, state.hue, state.saturation, bar);
}
