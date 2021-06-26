pixelCount = 864;

leader = 0;
direction = 1;
pixels = new Array(864);
hue = 0;
saturation = 1;

// fade default .0007
var speed = pixelCount / 10;
var fade = 0.0;

var fadeup = 0.01;

function beforeRender(delta) {
    if (fade > 1.0) {
        fade = 0.0;
    }
    fade += fadeup;
}

function render(index) {
    let m = (1 / 864) * index;
    hsv(index, 0.4, m, fade);
}
