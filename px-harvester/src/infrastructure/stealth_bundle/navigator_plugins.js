const fakePlugins = [
  { name: 'PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
  { name: 'Chrome PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
  { name: 'Chromium PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' }
];
Object.defineProperty(navigator, 'plugins', {
  get: () => fakePlugins,
  configurable: true
});
Object.defineProperty(navigator, 'mimeTypes', {
  get: () => [{ type: 'application/pdf', suffixes: 'pdf', description: 'Portable Document Format' }],
  configurable: true
});
