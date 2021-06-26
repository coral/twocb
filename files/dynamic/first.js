pixelCount = 864;

leader = 0;
direction = 1;
pixels = new Array(864);
hue = 0;
saturation = 1;

// fade default .0007
var speed = pixelCount / 10;
var fade = 1.0;

function beforeRender(delta) {}

function render(index) {
    let m = (1 / 864) * index;
    hsv(index, m, 1.0, 1.0);
}
