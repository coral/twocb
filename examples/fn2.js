function register() {
  return {
    parameters: ["hello", "hello2", "friends"],
  };
}
function beforeRender() {}

function render(a) {
  return a + 3.0;
}
