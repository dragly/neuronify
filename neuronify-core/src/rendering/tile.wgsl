struct Globals {
    view_matrix: mat4x4<f32>,
    transform: mat4x4<f32>,
    camera_center: vec4<f32>,
    camera_view_vector: vec4<f32>,
    camera_position: vec4<f32>,
    camera_up: vec4<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_size: vec4<f32>,
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> u_globals: Globals;

struct Tile {
    position: vec3<f32>,
    half_width: f32,
    half_spacing: f32,
    power: f32,
    area: f32,
    shading: f32,
    color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) shape_coord: vec2<f32>,
    @location(1) cell_coord: vec2<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) power: f32,
    @location(5) average: f32,
    @location(6) shading: f32,
    @location(7) step: f32,
};

fn tile_vertex(quad_coord: vec2<f32>, tile: Tile) -> VertexOutput {
    var output: VertexOutput;
    let offset = quad_coord * tile.half_spacing;
    let world_position = tile.position + vec3<f32>(offset.x, 0.0, offset.y);
    output.proj_position = u_globals.transform * vec4<f32>(world_position, 1.0);
    output.shape_coord = offset / tile.half_width;
    output.cell_coord = quad_coord;
    output.world_position = world_position;
    output.color = tile.color.rgb;
    output.power = tile.power;
    let cell_area = 4.0 * tile.half_spacing * tile.half_spacing;
    output.average = tile.area * tile.half_width * tile.half_width / cell_area;
    output.shading = tile.shading;
    output.step = 2.0 * tile.half_spacing / tile.half_width;
    return output;
}

@vertex
fn vs_main(
    @location(0) quad_coord: vec2<f32>,
) -> VertexOutput {
    var tile: Tile;
    return tile_vertex(quad_coord, tile);
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

const UP: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const SHARP_FROM_PIXELS: f32 = 2.0;
const SHARP_AT_PIXELS: f32 = 5.0;

fn coverage(shape_coord: vec2<f32>, power: f32) -> f32 {
    let distance = abs(shape_coord);
    let sum = pow(distance.x, power) + pow(distance.y, power);
    let radius = pow(max(sum, 1e-12), 1.0 / power);
    return clamp(0.5 + (1.0 - radius) / max(fwidth(radius), 1e-6), 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let lit = visula_directional_lit_vec3(in.color, UP, in.world_position);
    let color = mix(in.color, lit, in.shading);

    let side = select(vec2<f32>(-1.0), vec2<f32>(1.0), in.cell_coord >= vec2<f32>(0.0));
    let across = side * in.step;
    let edge = min(
        coverage(in.shape_coord, in.power)
            + coverage(in.shape_coord - vec2<f32>(across.x, 0.0), in.power)
            + coverage(in.shape_coord - vec2<f32>(0.0, across.y), in.power)
            + coverage(in.shape_coord - across, in.power),
        1.0,
    );

    let cell_footprint = max(fwidth(in.cell_coord.x), fwidth(in.cell_coord.y));
    let pixels_per_cell = 2.0 / max(cell_footprint, 1e-6);
    let sharpness = smoothstep(SHARP_FROM_PIXELS, SHARP_AT_PIXELS, pixels_per_cell);
    let alpha = mix(in.average, edge, sharpness);

    if alpha < 0.01 {
        discard;
    }

    var output: FragmentOutput;
    output.color = vec4<f32>(color, alpha);
    output.normal = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    return output;
}
