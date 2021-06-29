pixelCount = 864;

leader = 0;
direction = 1;
pixels = new Array(864);
hue = 0;
saturation = 1;

state = {
    speed: 0.001,
};

// fade default .0007
var speed = pixelCount / 10;
var fade = 0.0;

var fadeup = 0.01;
var hueup = 0.01;

var hue = 0.0;
var phase = 0.0;

function beforeRender(frame, delta) {
    // if (fade > 1.0) {
    //     fade = 0.0;
    // }
    // fade += state.speed;
    // if (hue > 1.0) {
    //     hue = 0.0;
    // }
    // hue += state.speed;
    //
    fade = sin(frame.phase, 0.5);
    phase = sin(frame.phase, 1);
    hue = 0.1;
}

function render3D(index, x, y, z) {
    let m = (1 / 864) * index;
    if (z > phase) {
        hsv(index, hue, m, fade);
    } else {
        hsv(index, 0.0, 0.0, 0.0);
    }
}
