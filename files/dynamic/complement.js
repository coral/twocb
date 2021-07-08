// Animation speed in milliseconds. Higher numbers are slower
state = {
    speed: 100,
};
var base_speed = 100000;
var speed = 10000;
// Percent of color wheel for ends. 1/2 is complementary, 1/3 and 2/3 is triadic, etc
var colorPercent = 1 / 2;
// Percent to dim the center blend. Higher numbers are darker.
var percentDim = 0.7;

var position = 0;
var rgb1 = [0.0, 0.0, 0.0];
var rgb2 = [0.0, 0.0, 0.0];

function rgb2hsv(h, s, v) {
    while (h < 0) h++;
    while (h >= 1) h--;
    if (s == 0) {
        rgb1[0] = v;
        rgb1[1] = v;
        rgb1[2] = v;
    } else {
        h *= 6;
        i = floor(h);
        var d0 = v * (1 - s);
        var d1 = v * (1 - s * (h - i));
        var d2 = v * (1 - s * (1 - (h - i)));
        if (i == 0) {
            rgb1[0] = v;
            rgb1[1] = d2;
            rgb1[2] = d0;
        } else if (i == 1) {
            rgb1[0] = d1;
            rgb1[1] = v;
            rgb1[2] = d0;
        } else if (i == 2) {
            rgb1[0] = d0;
            rgb1[1] = v;
            rgb1[2] = d2;
        } else if (i == 3) {
            rgb1[0] = d0;
            rgb1[1] = d1;
            rgb1[2] = v;
        } else if (i == 4) {
            rgb1[0] = d2;
            rgb1[1] = d0;
            rgb1[2] = v;
        } else {
            rgb1[0] = v;
            rgb1[1] = d0;
            rgb1[2] = d1;
        }
    }
}

function beforeRender(y, delta) {
    speed = (base_speed * state.speed ) + 1;
    position += delta / speed;
    if (position > 1) {
        position = 0;
    }
}

function render3D(index, x, y, z) {
    // Calculate the start hue and save the RGB
    rgb2hsv(position, 1, 1);
    for (var i = 0; i < 3; i++) {
        rgb2[i] = rgb1[i];
    }

    // Calculate the end hue
    rgb2hsv(position + colorPercent, 1, 1);

    // Determine the blended color at the z value
    for (var i = 0; i < 3; i++) {
        rgb1[i] = rgb2[i] + (rgb1[i] - rgb2[i]) * z;
    }

    // Apply dimming
    for (var i = 0; i < 3; i++) {
        if (z < 0.5) {
            rgb1[i] *= 1 - z * 2 * percentDim;
        } else {
            rgb1[i] *= 1 - percentDim + (z - 0.5) * (z * 2 * percentDim);
        }
    }

    rgb(index, rgb1[0], rgb1[1], rgb1[2]);
}
