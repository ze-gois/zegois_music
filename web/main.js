import init, { Synth, WaveformVisualizer } from "./pkg/zegois_music.js";

const playButton = document.querySelector("#play");
const stopButton = document.querySelector("#stop");
const bpmInput = document.querySelector("#bpm");
const bpmValue = document.querySelector("#bpmValue");

let audioContext;
let source;
let synth;
let visualizer;
let currentSamples = new Float32Array();
let animationFrame;
let startedAt = 0;

await init();
synth = new Synth();
visualizer = new WaveformVisualizer("visualizer");
visualizer.draw_idle();

bpmInput.addEventListener("input", () => {
  bpmValue.textContent = bpmInput.value;
  synth.set_bpm(Number(bpmInput.value));
});

playButton.addEventListener("click", async () => {
  stopCurrentSource();

  audioContext = audioContext ?? new AudioContext();
  await audioContext.resume();

  synth.set_bpm(Number(bpmInput.value));
  currentSamples = synth.render_melody(audioContext.sampleRate);

  const buffer = audioContext.createBuffer(1, currentSamples.length, audioContext.sampleRate);
  buffer.copyToChannel(currentSamples, 0);

  source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.onended = () => {
    stopButton.disabled = true;
    playButton.disabled = false;
    cancelAnimationFrame(animationFrame);
    visualizer.draw_waveform(currentSamples, 1);
  };

  startedAt = audioContext.currentTime;
  source.start();

  playButton.disabled = true;
  stopButton.disabled = false;
  animate();
});

stopButton.addEventListener("click", () => {
  stopCurrentSource();
  cancelAnimationFrame(animationFrame);
  visualizer.draw_waveform(currentSamples, 0);
});

function stopCurrentSource() {
  if (!source) return;

  source.onended = null;
  try {
    source.stop();
  } catch {
    // The source may already have ended; that is fine for this MVP.
  }
  source = null;
  playButton.disabled = false;
  stopButton.disabled = true;
}

function animate() {
  if (!audioContext || !currentSamples.length) return;

  const duration = currentSamples.length / audioContext.sampleRate;
  const progress = Math.min((audioContext.currentTime - startedAt) / duration, 1);
  visualizer.draw_waveform(currentSamples, progress);

  if (progress < 1) {
    animationFrame = requestAnimationFrame(animate);
  }
}


