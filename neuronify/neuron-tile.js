import ICON from "./neuron-icon.js";

function rings() {
  return ICON.rings.map((steps) => {
    const points = [];
    let x = 0;
    let y = 0;
    for (let i = 0; i < steps.length; i += 2) {
      x += steps[i] / 4;
      y += steps[i + 1] / 4;
      points.push([x, y]);
    }
    return points;
  });
}

function measure(tile) {
  const dpr = window.devicePixelRatio || 1;
  const area = tile.getBoundingClientRect();
  const square = tile.querySelector(".icon").getBoundingClientRect();
  const units = 1024 / square.width;
  return {
    width: Math.max(1, Math.round(area.width * dpr)),
    height: Math.max(1, Math.round(area.height * dpr)),
    view: [(square.left - area.left) * dpr, (square.top - area.top) * dpr, units / dpr],
  };
}

function drawStill(canvas, layout) {
  canvas.width = layout.width;
  canvas.height = layout.height;
  const [left, top, units] = layout.view;
  const context = canvas.getContext("2d");
  context.setTransform(1 / units, 0, 0, 1 / units, left, top);
  const ink = context.createLinearGradient(0, ICON.top, 0, ICON.bottom);
  ink.addColorStop(0, ICON.ink[0]);
  ink.addColorStop(1, ICON.ink[1]);
  context.fillStyle = ink;
  context.shadowColor = ICON.shadow + Math.round(255 * ICON.shadowOpacity).toString(16).padStart(2, "0");
  context.shadowBlur = (2 * ICON.shadowBlur) / units;
  context.shadowOffsetY = ICON.shadowDrop / units;
  const path = new Path2D();
  for (const points of rings()) {
    points.forEach(([x, y], i) => (i ? path.lineTo(x, y) : path.moveTo(x, y)));
    path.closePath();
  }
  context.fill(path, "evenodd");
}

export default function () {
  const tile = document.querySelector("#loading .tile");
  if (!tile) return;
  const canvas = document.getElementById("neuron");
  new ResizeObserver(() => drawStill(canvas, measure(tile))).observe(tile);
  canvas.classList.add("shown");
}
