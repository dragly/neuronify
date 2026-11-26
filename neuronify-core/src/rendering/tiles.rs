use visula::{
    Expression, InstanceBuffer, QuadPipeline, QuadPipelineDescriptor, RenderingDescriptor,
};
use visula_derive::Delegate;

use crate::rendering::gpu_types::TileData;
use crate::rendering::lighting::SHADING;

pub struct TileShape {
    pub power: f32,
    pub spacing: f32,
}

pub fn superellipse_area(power: f32) -> f32 {
    let steps = 10_000;
    let width = 1.0 / steps as f32;
    let quarter: f32 = (0..steps)
        .map(|step| {
            let x = (step as f32 + 0.5) * width;
            (1.0 - x.powf(power)).powf(1.0 / power) * width
        })
        .sum();
    4.0 * quarter
}

#[derive(Delegate)]
struct TileDelegate {
    position: Expression,
    half_width: Expression,
    half_spacing: Expression,
    power: Expression,
    area: Expression,
    shading: Expression,
    color: Expression,
}

pub fn create_tiles(
    rendering_descriptor: &RenderingDescriptor,
    tile_buffer: &InstanceBuffer<TileData>,
    shape: TileShape,
) -> Result<QuadPipeline, visula_core::ShaderError> {
    let tile = tile_buffer.instance();
    let corners: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
    QuadPipeline::new(
        rendering_descriptor,
        &QuadPipelineDescriptor {
            label: "tiles",
            shader_source: include_str!("tile.wgsl"),
            shader_variable_name: "tile",
            fragment_shader_variable_name: None,
            shadow_shader_source: None,
            vertex_data: bytemuck::cast_slice(&corners),
            vertex_stride: size_of::<[f32; 2]>(),
            vertex_format: wgpu::VertexFormat::Float32x2,
            index_data: bytemuck::cast_slice(&indices),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group_layouts: &[],
            depth_write_enabled: true,
            depth_bias: wgpu::DepthBiasState::default(),
        },
        &TileDelegate {
            position: tile.position,
            half_width: tile.half_width,
            half_spacing: (shape.spacing / 2.0).into(),
            power: shape.power.into(),
            area: superellipse_area(shape.power).into(),
            shading: SHADING.into(),
            color: tile.color,
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_superellipse_of_power_two_is_a_circle() {
        assert!((superellipse_area(2.0) - std::f32::consts::PI).abs() < 1e-3);
    }
}
