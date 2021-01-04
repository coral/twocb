function register() {
    return {
        parameters: ["hello", "hello2", "friends"],
        features: ["fft", "colorchord", "tempo"],
    };
}

function beforeRender() {}

function render(m) {
    rgb(m, 1.0, 0.9, 0.2);
}
