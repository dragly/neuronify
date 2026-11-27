import ICON from "./neuron-icon.js";

const WAVELENGTHS = 16;
const PERIOD = 2.8;
const MILD = 0.75;
const FIELD_SIZE = 768;
const TRACE_PAD = 140;
const TRACE_TEXEL = 1.5;
const REVEAL_MS = 240;

function vec3(hex) {
  const value = parseInt(hex.slice(1), 16);
  return `vec3f(${[16, 8, 0].map((shift) => (((value >> shift) & 255) / 255).toFixed(5)).join(", ")})`;
}

const FULLSCREEN = `
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let corner = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4f(corner * 2.0 - 1.0, 0.0, 1.0);
}
`;

const NEURON_BAKE = `
@group(0) @binding(0) var<storage, read> edges: array<vec4f>;
@group(0) @binding(1) var<uniform> size: vec4f;
${FULLSCREEN}
@fragment
fn neuron_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let p = position.xy * (1024.0 / size.x);
  var nearest = 1e12;
  var inside = false;
  for (var i = 0u; i < arrayLength(&edges); i++) {
    let edge = edges[i];
    let a = edge.xy;
    let ab = edge.zw - a;
    let ap = p - a;
    let offset = ap - ab * clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
    nearest = min(nearest, dot(offset, offset));
    if (a.y > p.y) != (edge.w > p.y) && p.x < ab.x * (p.y - a.y) / ab.y + a.x {
      inside = !inside;
    }
  }
  let dist = sqrt(nearest);
  return vec4f(select(dist, -dist, inside), 0.0, 0.0, 1.0);
}
`;

const TRACE_BAKE = `
@group(0) @binding(0) var<storage, read> points: array<vec2f>;
@group(0) @binding(1) var<uniform> area: vec4f;
${FULLSCREEN}
@fragment
fn trace_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let p = area.xy + position.xy * area.z;
  let pad = area.w;
  let count = arrayLength(&points);
  var low = 0u;
  var high = count - 1u;
  while low < high {
    let middle = (low + high) / 2u;
    if points[middle].x < p.x - pad {
      low = middle + 1u;
    } else {
      high = middle;
    }
  }
  var nearest = pad * pad;
  for (var i = max(low, 1u) - 1u; i + 1u < count; i++) {
    let a = points[i];
    if a.x > p.x + pad {
      break;
    }
    let ab = points[i + 1u] - a;
    let ap = p - a;
    let offset = ap - ab * clamp(dot(ap, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
    nearest = min(nearest, dot(offset, offset));
  }
  return vec4f(sqrt(nearest), 0.0, 0.0, 1.0);
}
`;

const DRAW = `
struct Params {
  view: vec4f,
  tile: vec4f,
  plot: vec4f,
  trace_area: vec4f,
  divider: vec4f,
  marks: vec4f,
  spectrum: array<vec4f, ${WAVELENGTHS}>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var neuron_field: texture_2d<f32>;
@group(0) @binding(2) var trace_field: texture_2d<f32>;
@group(0) @binding(3) var field_sampler: sampler;

const SCALE: f32 = 0.08;
const SHIFT: f32 = 20.0;
const BLUR: f32 = 18.0;
const SHARP: f32 = 0.6;
const MARK_LENS: f32 = 0.3;
const DIVIDER_ALPHA: f32 = 0.3;
const TILE_TOP = ${vec3(ICON.tile[0])};
const TILE_BOTTOM = ${vec3(ICON.tile[1])};
const INK_TOP = ${vec3(ICON.ink[0])};
const INK_BOTTOM = ${vec3(ICON.ink[1])};
const SHADOW = ${vec3(ICON.shadow)};
const SHADOW_OPACITY: f32 = ${ICON.shadowOpacity};
const SHADOW_BLUR: f32 = ${ICON.shadowBlur};
const SHADOW_DROP: f32 = ${ICON.shadowDrop};
const INK_FROM: f32 = ${ICON.top};
const INK_TO: f32 = ${ICON.bottom};
${FULLSCREEN}
fn decode(c: vec3f) -> vec3f {
  return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}

fn encode(c: vec3f) -> vec3f {
  return select(1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}

fn cover(dist: f32, width: f32) -> f32 {
  return 1.0 - smoothstep(-width, width, dist);
}

fn stroke(dist: f32, half: f32, width: f32) -> f32 {
  return smoothstep(-width, width, dist + half) - smoothstep(-width, width, dist - half);
}

fn neuron_at(p: vec2f) -> f32 {
  return textureSampleLevel(neuron_field, field_sampler, p / 1024.0, 0.0).r;
}

fn trace_at(p: vec2f) -> f32 {
  let uv = (p - params.trace_area.xy) / params.trace_area.zw;
  if any(uv < vec2f(0.0)) || any(uv > vec2f(1.0)) {
    return 1e4;
  }
  return textureSampleLevel(trace_field, field_sampler, uv, 0.0).r;
}

fn divider_at(p: vec2f) -> f32 {
  let line = params.divider;
  return length(vec2f(max(max(line.x - p.x, p.x - line.z), 0.0), p.y - line.y));
}

fn revealed(x: f32, width: f32) -> f32 {
  let right = mix(params.plot.x, params.plot.z, params.marks.x);
  return max(cover(x - right, width) - cover(x - params.plot.x, width), 0.0);
}

fn ground(p: vec2f) -> vec3f {
  let t = clamp((p.y - params.tile.y) / (params.tile.w - params.tile.y), 0.0, 1.0);
  let shade = cover(neuron_at(p - vec2f(0.0, SHADOW_DROP)), 1.9 * SHADOW_BLUR) * SHADOW_OPACITY;
  return mix(mix(TILE_TOP, TILE_BOTTOM, t), SHADOW, shade);
}

fn seen(at: vec2f, width: f32, mark_at: vec2f, mark_width: f32) -> vec3f {
  let ink = mix(INK_TOP, INK_BOTTOM, clamp((at.y - INK_FROM) / (INK_TO - INK_FROM), 0.0, 1.0));
  let neuron = mix(ground(at), ink, cover(neuron_at(at), width));
  let trace = stroke(trace_at(mark_at), params.marks.y, mark_width) * revealed(mark_at.x, mark_width);
  let line = stroke(divider_at(mark_at), params.divider.w, mark_width) * DIVIDER_ALPHA;
  return decode(mix(neuron, vec3f(1.0), max(trace, line)));
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let units = params.view.z;
  let p = (position.xy - params.view.xy) * units;
  let radius = params.marks.z;
  let q = abs(p - 0.5 * (params.tile.xy + params.tile.zw)) - (0.5 * (params.tile.zw - params.tile.xy) - vec2f(radius));
  let edge = length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - radius;
  let alpha = clamp(0.5 - edge / units, 0.0, 1.0);
  if alpha <= 0.0 {
    return vec4f(0.0);
  }
  let s = params.view.w;
  let aa = 0.75 * units;
  let reach = s * (1.1 * SCALE * length(p - 512.0) + SHIFT) + 1.9 * (SHARP + BLUR * s) + aa;
  let inside = all(p > vec2f(0.0)) && all(p < vec2f(1024.0));
  let right = mix(params.plot.x, params.plot.z, params.marks.x);
  let near = (inside && neuron_at(p) <= reach + SHADOW_DROP + 3.0 * SHADOW_BLUR)
    || (p.x < right + reach && trace_at(p) <= reach)
    || divider_at(p) <= reach;
  var colour = vec3f(0.0);
  if !near {
    colour = decode(ground(p));
  } else if s < 0.001 {
    let width = max(aa, 1.9 * SHARP);
    colour = seen(p, width, p, width);
  } else {
    let m = MARK_LENS * s;
    let spacing = 2.0 / ${WAVELENGTHS}.0 * m * (SCALE * length(p - 512.0) + SHIFT);
    for (var i = 0u; i < ${WAVELENGTHS}u; i++) {
      let band = params.spectrum[i];
      let x = band.w;
      let at = 512.0 + (p - 512.0) / (1.0 + SCALE * s * x) + vec2f(SHIFT * s * x, 0.0);
      let mark_at = 512.0 + (p - 512.0) / (1.0 + SCALE * m * x) + vec2f(SHIFT * m * x, 0.0);
      let width = max(aa, 1.9 * (SHARP + BLUR * s * x * x));
      let mark_width = max(aa, 1.9 * SHARP + 0.8 * spacing + 1.9 * BLUR * m * x * x);
      colour += band.rgb * seen(at, width, mark_at, mark_width);
    }
  }
  return vec4f(encode(clamp(colour, vec3f(0.0), vec3f(1.0))) * alpha, alpha);
}
`;

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

function edges() {
  const values = [];
  for (const points of rings()) {
    points.forEach(([x0, y0], i) => {
      const [x1, y1] = points[(i + 1) % points.length];
      values.push(x0, y0, x1, y1);
    });
  }
  return new Float32Array(values);
}

function spectrum() {
  const lobe = (l, mu, below, above) => Math.exp(-0.5 * ((l - mu) / (l < mu ? below : above)) ** 2);
  const rows = [];
  for (let i = 0; i < WAVELENGTHS; i++) {
    const l = 400 + (300 * (i + 0.5)) / WAVELENGTHS;
    const x = 1.056 * lobe(l, 599.8, 37.9, 31.0) + 0.362 * lobe(l, 442.0, 16.0, 26.7) - 0.065 * lobe(l, 501.1, 20.4, 26.2);
    const y = 0.821 * lobe(l, 568.8, 46.9, 40.5) + 0.286 * lobe(l, 530.9, 16.3, 31.1);
    const z = 1.217 * lobe(l, 437.0, 11.8, 36.0) + 0.681 * lobe(l, 459.0, 26.0, 13.8);
    rows.push([
      Math.max(0, 3.2406 * x - 1.5372 * y - 0.4986 * z),
      Math.max(0, -0.9689 * x + 1.8758 * y + 0.0415 * z),
      Math.max(0, 0.0557 * x - 0.204 * y + 1.057 * z),
      (l - 550) / 150,
    ]);
  }
  const sums = [0, 1, 2].map((c) => rows.reduce((sum, row) => sum + row[c], 0));
  return rows.flatMap(([r, g, b, x]) => [r / sums[0], g / sums[1], b / sums[2], x]);
}

function eased(from, to, t) {
  const u = Math.min(1, Math.max(0, t));
  return from + (to - from) * u * u * (3 - 2 * u);
}

function pulse(phase) {
  if (phase < 0.25) return 0;
  if (phase < 0.45) return eased(0, MILD, (phase - 0.25) / 0.2);
  if (phase < 0.6) return MILD;
  if (phase < 0.72) return eased(MILD, 1, (phase - 0.6) / 0.12);
  return eased(1, 0, (phase - 0.72) / 0.28);
}

function pulses() {
  const cycle = () => 1000 * PERIOD * (0.5 + Math.random());
  let start = performance.now();
  let length = cycle();
  return (now) => {
    while (now - start >= length) {
      start += length;
      length = cycle();
    }
    return pulse(Math.max(0, now - start) / length);
  };
}

function measure(tile) {
  const dpr = window.devicePixelRatio || 1;
  const area = tile.getBoundingClientRect();
  const square = tile.querySelector(".icon").getBoundingClientRect();
  const graph = tile.querySelector(".plot").getBoundingClientRect();
  const line = tile.querySelector(".status").getBoundingClientRect();
  const units = 1024 / square.width;
  const at = (x, y) => [(x - square.left) * units, (y - square.top) * units];
  return {
    width: Math.max(1, Math.round(area.width * dpr)),
    height: Math.max(1, Math.round(area.height * dpr)),
    view: [(square.left - area.left) * dpr, (square.top - area.top) * dpr, units / dpr],
    tile: [...at(area.left, area.top), ...at(area.right, area.bottom)],
    radius: parseFloat(getComputedStyle(tile).borderTopLeftRadius) * units,
    plot: [...at(graph.left, graph.top), ...at(graph.right, graph.bottom)],
    divider: [...at(line.left, line.top + 0.5), at(line.right, line.top)[0], 0.5 * units],
    halfStroke: units,
  };
}

function tracePoints(plot) {
  const svg = document.querySelector("#trace svg");
  const { width, height } = svg.viewBox.baseVal;
  const values = svg.querySelector("path").getAttribute("d").match(/-?[\d.]+/g).map(Number);
  const [x0, y0, x1, y1] = plot;
  return Float32Array.from(values, (value, i) => (i % 2 ? y0 + (value / height) * (y1 - y0) : x0 + (value / width) * (x1 - x0)));
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

async function startLens(canvas, onFirstFrame) {
  const adapter = await navigator.gpu?.requestAdapter({ powerPreference: "low-power" });
  if (!adapter) return null;
  const device = await adapter.requestDevice();
  const format = navigator.gpu.getPreferredCanvasFormat();
  const context = canvas.getContext("webgpu");
  context.configure({ device, format, alphaMode: "premultiplied" });
  const pipelineFor = (code, entryPoint, target) => {
    const module = device.createShaderModule({ code });
    return device.createRenderPipelineAsync({
      layout: "auto",
      vertex: { module, entryPoint: "vs_main" },
      fragment: { module, entryPoint, targets: [{ format: target }] },
    });
  };
  const [neuronPipeline, tracePipeline, drawPipeline] = await Promise.all([
    pipelineFor(NEURON_BAKE, "neuron_main", "r16float"),
    pipelineFor(TRACE_BAKE, "trace_main", "r16float"),
    pipelineFor(DRAW, "fs_main", format),
  ]);
  const buffer = (data, usage) => {
    const created = device.createBuffer({ size: data.byteLength, usage: usage | GPUBufferUsage.COPY_DST });
    device.queue.writeBuffer(created, 0, data);
    return created;
  };
  const field = (width, height) =>
    device.createTexture({ size: [width, height], format: "r16float", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING });
  const bake = (target, pipeline, buffers) => {
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({ colorAttachments: [{ view: target.createView(), loadOp: "clear", storeOp: "store" }] });
    pass.setPipeline(pipeline);
    pass.setBindGroup(
      0,
      device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: buffers.map((data, binding) => ({ binding, resource: { buffer: data } })),
      }),
    );
    pass.draw(3);
    pass.end();
    device.queue.submit([encoder.finish()]);
  };

  const neuronField = field(FIELD_SIZE, FIELD_SIZE);
  bake(neuronField, neuronPipeline, [
    buffer(edges(), GPUBufferUsage.STORAGE),
    buffer(new Float32Array([FIELD_SIZE, 0, 0, 0]), GPUBufferUsage.UNIFORM),
  ]);
  const values = new Float32Array(24 + 4 * WAVELENGTHS);
  values.set(spectrum(), 24);
  const uniforms = device.createBuffer({ size: values.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  const sampler = device.createSampler({ magFilter: "linear", minFilter: "linear" });
  let traceField = field(1, 1);
  let traceKey = "";
  let bindGroup = null;

  const still = matchMedia("(prefers-reduced-motion: reduce)");
  const strengthAt = pulses();
  let reveal = { from: 0, to: 0, at: 0 };
  const revealedAt = (now) => (still.matches ? reveal.to : reveal.from + (reveal.to - reveal.from) * Math.min(1, (now - reveal.at) / REVEAL_MS));
  let drawn = [-1, -1];
  let frame = 0;
  const render = (now, always) => {
    const strength = still.matches ? 0 : strengthAt(now);
    const shown = revealedAt(now);
    if (!bindGroup || (!always && Math.abs(strength - drawn[0]) < 0.001 && Math.abs(shown - drawn[1]) < 1e-4)) return;
    drawn = [strength, shown];
    values[3] = strength;
    values[20] = shown;
    device.queue.writeBuffer(uniforms, 0, values);
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ view: context.getCurrentTexture().createView(), loadOp: "clear", storeOp: "store" }],
    });
    pass.setPipeline(drawPipeline);
    pass.setBindGroup(0, bindGroup);
    pass.draw(3);
    pass.end();
    device.queue.submit([encoder.finish()]);
    onFirstFrame?.();
    onFirstFrame = null;
  };
  const loop = (now) => {
    frame = requestAnimationFrame(loop);
    render(now, false);
  };
  frame = requestAnimationFrame(loop);
  device.lost.then(() => cancelAnimationFrame(frame));

  return {
    configure(layout) {
      if (canvas.width !== layout.width || canvas.height !== layout.height) {
        canvas.width = layout.width;
        canvas.height = layout.height;
      }
      const key = layout.plot.map((value) => value.toFixed(2)).join();
      if (key !== traceKey) {
        traceKey = key;
        const [x0, y0, x1, y1] = layout.plot;
        const columns = Math.ceil((x1 - x0 + 2 * TRACE_PAD) / TRACE_TEXEL);
        const rows = Math.ceil((y1 - y0 + 2 * TRACE_PAD) / TRACE_TEXEL);
        const area = [x0 - TRACE_PAD, y0 - TRACE_PAD, columns * TRACE_TEXEL, rows * TRACE_TEXEL];
        const spent = [traceField];
        traceField = field(columns, rows);
        spent.push(
          buffer(tracePoints(layout.plot), GPUBufferUsage.STORAGE),
          buffer(new Float32Array([area[0], area[1], TRACE_TEXEL, TRACE_PAD]), GPUBufferUsage.UNIFORM),
        );
        bake(traceField, tracePipeline, spent.slice(1));
        device.queue.onSubmittedWorkDone().then(() => spent.forEach((resource) => resource.destroy()));
        values.set(area, 12);
        bindGroup = device.createBindGroup({
          layout: drawPipeline.getBindGroupLayout(0),
          entries: [
            { binding: 0, resource: { buffer: uniforms } },
            { binding: 1, resource: neuronField.createView() },
            { binding: 2, resource: traceField.createView() },
            { binding: 3, resource: sampler },
          ],
        });
      }
      values.set(layout.view, 0);
      values.set(layout.tile, 4);
      values.set(layout.plot, 8);
      values.set(layout.divider, 16);
      values.set([layout.halfStroke, layout.radius], 21);
      render(performance.now(), true);
    },
    progress(fraction) {
      const now = performance.now();
      reveal = { from: revealedAt(now), to: fraction, at: now };
    },
    stop() {
      cancelAnimationFrame(frame);
      context.unconfigure();
      device.destroy();
    },
  };
}

export default async function () {
  const loading = document.getElementById("loading");
  const tile = loading?.querySelector(".tile");
  if (!tile) return null;
  let canvas = document.getElementById("neuron");
  const lens = await startLens(canvas, () => {
    canvas.classList.add("shown");
    tile.classList.add("lens");
  }).catch((error) => {
    console.error(error);
    const fresh = canvas.cloneNode();
    canvas.replaceWith(fresh);
    canvas = fresh;
    return null;
  });
  if (!lens) {
    new ResizeObserver(() => drawStill(canvas, measure(tile))).observe(tile);
    canvas.classList.add("shown");
    return null;
  }
  const resized = new ResizeObserver(() => lens.configure(measure(tile)));
  const stop = () => {
    resized.disconnect();
    lens.stop();
  };
  if (!loading.isConnected) {
    stop();
    return null;
  }
  resized.observe(tile);
  loading.addEventListener("transitionend", (event) => event.target === loading && stop());
  return lens;
}
