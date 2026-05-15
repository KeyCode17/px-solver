Object.defineProperty(Navigator.prototype, 'webdriver', {
  get: () => undefined,
  configurable: true
});
delete Object.getPrototypeOf(navigator).webdriver;
