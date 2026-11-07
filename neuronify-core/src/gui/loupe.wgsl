struct Loupe {
    center: vec2<f32>,
    source: vec2<f32>,
    size: vec2<f32>,
    radius: f32,
    zoom: f32,
    encode_srgb: u32,
    padding_a: u32,
    padding_b: vec2<u32>,
};

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> loupe: Loupe;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32(index & 1u), f32(index >> 1u)) * 2.0 - 1.0;
    return vec4<f32>(corner, 0.0, 1.0);
}

fn srgb_from_linear(x: vec3<f32>) -> vec3<f32> {
    let c = clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
    let low = c * 12.92;
    let high = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let offset = position.xy - loupe.center;
    let coverage = clamp(loupe.radius - length(offset) + 0.5, 0.0, 1.0);
    let uv = (loupe.source + offset / loupe.zoom) / loupe.size;
    var color = textureSampleLevel(scene, scene_sampler, uv, 0.0).rgb;
    if loupe.encode_srgb == 1u {
        color = srgb_from_linear(color);
    }
    return vec4<f32>(color, coverage);
}
