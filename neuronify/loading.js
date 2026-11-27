const RESTING_POTENTIAL = -65;
const LOWEST_SHOWN_POTENTIAL = -100;
const HIGHEST_SHOWN_POTENTIAL = 50;
const SAMPLES = 700;

function random(seed) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function actionPotential(delay) {
  if (delay < -0.02 || delay > 0.06) return 0;
  if (delay < 0) return 15 * Math.exp(delay / 0.006);
  if (delay < 0.0015) return 15 + 80 * (delay / 0.0015);
  if (delay < 0.004) return 95 - 108 * ((delay - 0.0015) / 0.0025);
  return -13 * Math.exp(-(delay - 0.004) / 0.012);
}

function synapticPotential(delay, size) {
  if (delay <= 0 || delay >= 0.04) return 0;
  return size * (delay / 0.005) * Math.exp(1 - delay / 0.005);
}

function membraneTrace() {
  const next = random(5);
  const spikes = [];
  for (let x = 0.06 + next() * 0.03; x < 0.96; x += 0.05 + next() * 0.075) spikes.push(x);
  const inputs = Array.from({ length: 46 }, () => [next(), 0.8 + next() * 3]);
  const potential = (x) =>
    Math.min(
      32,
      RESTING_POTENTIAL +
        0.6 * Math.sin(x * 1700) * Math.sin(x * 530) +
        inputs.reduce((sum, [at, size]) => sum + synapticPotential(x - at, size), 0) +
        spikes.reduce((sum, at) => sum + actionPotential(x - at), 0),
    );
  const height = (voltage) =>
    (100 * (HIGHEST_SHOWN_POTENTIAL - voltage)) / (HIGHEST_SHOWN_POTENTIAL - LOWEST_SHOWN_POTENTIAL);
  const xs = Array.from({ length: SAMPLES + 1 }, (_, i) => i / SAMPLES);
  for (const at of spikes) xs.push(at, at + 0.0015, at + 0.004);
  xs.sort((a, b) => a - b);
  return xs
    .map((x, i) => `${i ? "L" : "M"}${(x * 1000).toFixed(1)} ${height(potential(x)).toFixed(2)}`)
    .join("");
}

export default function () {
  const trace = document.getElementById("trace");
  const reveal = trace.parentElement;
  const percent = document.getElementById("percent");
  trace.innerHTML = `<svg viewBox="0 0 1000 100" preserveAspectRatio="none"><path d="${membraneTrace()}" /></svg>`;
  let shownPercent = 0;
  let fraction = 0;
  let lens = null;
  import("./neuron-tile.js")
    .then((tile) => tile.default())
    .then((started) => {
      lens = started;
      lens?.progress(fraction);
    });
  return {
    onProgress({ current, total }) {
      fraction = Math.min(1, current / total);
      lens?.progress(fraction);
      reveal.style.transform = `translateX(${(fraction - 1) * 100}%)`;
      trace.style.transform = `translateX(${(1 - fraction) * 100}%)`;
      const rounded = Math.floor(fraction * 100);
      if (rounded !== shownPercent) {
        shownPercent = rounded;
        percent.textContent = `${rounded} %`;
      }
    },
  };
}
