/*
 Bouncer 2D/3D
 
 Bounces (up to 20) balls objects around a 2D or 3D display. 
 Use sliders to set "ball" count, size and speed.  
 
 Requires a 2D or 3D LED array and appropriate pixel mapper.
 
 Version  Author        Date        Comment
 1.0.1    JEM(ZRanger1) 11/12/2020 
*/

// Global Variables
var maxBalls = 20;
var numBalls = 6;
var ballSize = 0.06;
var speed = 0.0075;
var ballSize3D = ballSize * 4;
var h = 0.0;

// array of ball vectors
var balls = new Array(maxBalls);

// UI
function sliderBalls(v) {
    numBalls = floor(1 + v * (maxBalls - 1));
}

function sliderSize(v) {
    ballSize = 0.2 * v;
    ballSize3D = ballSize * 4;
}

function sliderSpeed(v) {
    speed = 0.15 * v;
    initBalls();
}

// allocate memory for ball vectors
function createBalls() {
    for (var i = 0; i < maxBalls; i++) {
        balls[i] = new Array(7);
    }
}

// create ball vector with a random position, direction, speed, color
function initBalls() {
    for (var i = 0; i < numBalls; i++) {
        var b = balls[i];

        b[0] = random(1); // x pos
        b[1] = random(1); // y pos
        b[2] = random(1); // z pos

        b[3] = random(speed); // x vec
        b[4] = random(speed); // y vec
        b[5] = random(speed); // z vec

        b[6] = random(1); // hue
    }
}

// move balls and bounce them off "walls"
function bounce() {
    for (var i = 0; i < numBalls; i++) {
        var b = balls[i];

        // move ball
        b[0] += b[3];
        b[1] += b[4];
        b[2] += b[5];

        // bounce off walls by flipping vector element sign when we hit.
        // If we do hit a wall, we exit early, trading precision
        // in corners for speed.  We'll catch it in a frame or two anyway
        if (b[0] < 0) {
            b[0] = 0;
            b[3] = -b[3];
            continue;
        }
        if (b[1] < 0) {
            b[1] = 0;
            b[4] = -b[4];
            continue;
        }
        if (b[2] < 0) {
            b[2] = 0;
            b[5] = -b[5];
            continue;
        }

        if (b[0] > 1) {
            b[0] = 1;
            b[3] = -b[3];
            continue;
        }
        if (b[1] > 1) {
            b[1] = 1;
            b[4] = -b[4];
            continue;
        }
        if (b[2] > 1) {
            b[2] = 1;
            b[5] = -b[5];
            continue;
        }
    }
}

createBalls();
initBalls();

function beforeRender(delta) {
    bounce();
}

function render3D(index, x, y, z) {
    var dx, dy, dz;
    var v = 0;
    var s = 1;
    h = 0.0;

    for (var i = 0; i < numBalls; i++) {
        if ((dx = abs(balls[i][0] - x)) > ballSize3D) continue;
        if ((dy = abs(balls[i][1] - y)) > ballSize3D) continue;
        if ((dz = abs(balls[i][2] - z)) <= ballSize3D) {
            v = (dx + dy + dz) / ballSize3D;
            v = v * v;
            s = v * 4;
            v = 1 - v;
            h = balls[i][6];
            break;
        }
    }

    hsv(index, h, s, v);
}
