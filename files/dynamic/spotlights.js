scale = 1 / (PI * PI); // How large the "spotlights" are
speed = 1; // How fast things move around
phase = 0.0;

function beforeRender(delta, frame) {
    phase = frame.phase;

    t1 = 2 * triangle(time(0.03 / speed)) - 1;
    t2 = 2 * triangle(time(0.04 / speed)) - 1;
    t3 = 2 * triangle(time(0.05 / speed)) - 1;
    t4 = time(0.02 / speed);

    // The axis we'll rotate around is a vector (t1, t2, t3) - each -1..1
    // The angle to rotate about it is just a 0..2*PI sawtooth
    setupRotationMatrix(t1, t2, t3, t4 * PI2);
}

function render3D(index, _x, _y, _z) {
    // Shift (0, 0, 0) to be the center of the world, not the rear-top-left
    x = _x - 0.5;
    y = _y - 0.5;
    z = _z - 0.5;

    // In befreRender() we calculated a rotation matrix for this frame
    // rotate3D() now applies it to the current pixel's position
    rotate3D(x, y, z);
    x = rx;
    y = ry;
    z = rz; // clunky way we adopt the optput of rotate3D()

    // dist is the distance (in world units) from a cone's surface to this pixel
    // Positive values are inside the cone
    // If you try a different scale for x vs y, you'll see elliptical cones
    dist = abs(z) - sqrt((x * x) / scale + (y * y) / scale);

    dist = clamp(dist, -1, 1); // Try commenting this out

    //  magenta,  white center,  sub-pixel rendered border
    hsv(index, 0.97, 1 - dist, pow(1 + dist, 4));
}

/*
  setupRotationMatrix()
  Takes a vector (ux, uy, uz) which will be the axis to rotate around
    and an angle in radians
  It computes a 3D rotation matrix and stores it in a global named R
  
  https://en.wikipedia.org/wiki/Rotation_matrix
*/

var R = new Array(3);
for (i = 0; i < 3; i++) R[i] = new Array(3); // init 3x3, R[r][c]

function setupRotationMatrix(ux, uy, uz, angle) {
    // Rescale ux, uy, uz to make sure it's a unit vector, length = 1
    length = sqrt(ux * ux + uy * uy + uz * uz);
    ux /= length;
    uy /= length;
    uz /= length;

    // Precompute a few reused values
    cosa = cos(angle);
    sina = sin(angle);
    ccosa = 1 - cosa;
    xyccosa = ux * uy * ccosa;
    xzccosa = ux * uz * ccosa;
    yzccosa = uy * uz * ccosa;
    xsina = ux * sina;
    ysina = uy * sina;
    zsina = uz * sina;

    R[0][0] = cosa + ux * ux * ccosa;
    R[0][1] = xyccosa - zsina;
    R[0][2] = xzccosa + ysina;
    R[1][0] = xyccosa + zsina;
    R[1][1] = cosa + uy * uy * ccosa;
    R[1][2] = yzccosa - xsina;
    R[2][0] = xzccosa - ysina;
    R[2][1] = yzccosa + xsina;
    R[2][2] = cosa + uz * uz * ccosa;
}

/*
  rotate3D()
  Takes 3 coordinates (x, y, z) and expects R to be a global rotation matrix.
  Sets globals rx, ry, and rz as the rotated point's new coordinates.
  (Globals are used for speed and convenience in the Pixelblaze lang)
*/
var rx, ry, rz;
function rotate3D(x, y, z) {
    rx = R[0][0] * x + R[0][1] * y + R[0][2] * z;
    ry = R[1][0] * x + R[1][1] * y + R[1][2] * z;
    rz = R[2][0] * x + R[2][1] * y + R[2][2] * z;
}
