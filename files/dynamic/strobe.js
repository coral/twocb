state = {
    hue: 1.0,
    saturation: 0.0,
    speed: 0.0,
};

var bar = 0.0;

function beforeRender(frame, delta) {
    if (frame.bar > 0.9) {
        bar = square(frame.phase, state.speed);
    } else {
        bar = 0.0;
    }

}

function render(index) {
    hsv(index, state.hue, state.saturation, bar);
}
