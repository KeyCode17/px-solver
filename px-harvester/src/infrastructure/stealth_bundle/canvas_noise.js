const origGetImageData = CanvasRenderingContext2D.prototype.getImageData;
CanvasRenderingContext2D.prototype.getImageData = function(sx, sy, sw, sh) {
  const data = origGetImageData.call(this, sx, sy, sw, sh);
  for (let i = 0; i < data.data.length; i += 4) {
    data.data[i] = (data.data[i] + 1) & 0xff;
  }
  return data;
};
const origToDataURL = HTMLCanvasElement.prototype.toDataURL;
HTMLCanvasElement.prototype.toDataURL = function(...args) {
  const ctx = this.getContext('2d');
  if (ctx) {
    const w = this.width, h = this.height;
    if (w > 0 && h > 0) {
      const d = origGetImageData.call(ctx, 0, 0, w, h);
      d.data[0] = (d.data[0] + 1) & 0xff;
      ctx.putImageData(d, 0, 0);
    }
  }
  return origToDataURL.apply(this, args);
};
