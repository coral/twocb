function register() {
    return {
        parameters: ["hello", "hello2", "friends"],
        features: ["fft", "colorchord", "tempo"],
    };
}

function beforeRender() {}

function render(index) {
    return 3.0;
}
