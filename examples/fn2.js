function register() {
    return {
        parameters: ["hello", "hello2", "friends"],
        features: ["fft", "colorchord", "tempo"],
    };
}

// function beforeRender() {}

// function render(m) {
//     rgb(m.I, 1.0, 0.9, 0.2);
// }

pixelCount = 768;

leader = 0;
direction = 1;
pixels = new Array(768);
hue = 0;
saturation = 1;

// fade default .0007
var speed = pixelCount / 600;
var fade = 0.001;

function beforeRender(delta) {
    leader += direction * delta * speed;
    if (leader >= pixelCount) {
        direction = -direction;
        leader = pixelCount - 1;
    }

    if (leader < 0) {
        direction = -direction;
        leader = 0;
    }
    pixels[floor(leader)] = 1;
    for (i = 0; i < pixelCount; i++) {
        pixels[i] -= delta * fade;
        pixels[i] = max(0, pixels[i]);
    }
}

function render(index) {
    v = pixels[index];
    v = v * v * v;
    hsv(index, hue, saturation, v);
}
