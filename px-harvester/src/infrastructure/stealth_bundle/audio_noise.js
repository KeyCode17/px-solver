if (window.AudioBuffer) {
  const origGetChannelData = AudioBuffer.prototype.getChannelData;
  AudioBuffer.prototype.getChannelData = function(channel) {
    const data = origGetChannelData.call(this, channel);
    for (let i = 0; i < data.length; i += 100) {
      data[i] = data[i] + (Math.random() - 0.5) * 0.0000001;
    }
    return data;
  };
}
