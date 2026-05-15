if (!window.chrome) {
  Object.defineProperty(window, 'chrome', { value: {}, configurable: true });
}
if (!window.chrome.runtime) {
  Object.defineProperty(window.chrome, 'runtime', { value: {}, configurable: true });
}
