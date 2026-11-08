use glam::Vec3;
use visula::primitives::mesh_primitive::MeshVertexAttributes;
use visula::{
    Expression, InstanceBuffer, MeshGeometry, MeshMaterial, MeshPipeline, RenderingDescriptor,
};

use crate::components::*;
use crate::measurement::voltmeter::Frame;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::MeterBackgroundData;
use hecs::Entity;

const METER_BACKGROUND_DROP: f32 = 0.1;

pub fn create_meter_backgrounds(
    rendering_descriptor: &RenderingDescriptor,
    device: &wgpu::Device,
    background_buffer: &InstanceBuffer<MeterBackgroundData>,
    palette: &Palette,
) -> Result<MeshPipeline, visula_core::ShaderError> {
    let background = background_buffer.instance();
    let mut backgrounds = MeshPipeline::new(
        rendering_descriptor,
        &MeshGeometry {
            position: background.position,
            scale: background.scale,
            ..Default::default()
        },
        &MeshMaterial {
            color: Expression::from(palette.meter_background.extend(1.0)).directional_lit(),
        },
    )?;
    let corner = |x: f32, z: f32| MeshVertexAttributes {
        position: [x, 0.0, z],
        normal: [0.0, 1.0, 0.0],
        uv: [x, z],
        color: [255; 4],
    };
    let vertices = [
        corner(0.0, 0.0),
        corner(1.0, 0.0),
        corner(0.0, 1.0),
        corner(1.0, 1.0),
    ];
    backgrounds.set_mesh_data(device, &vertices, &[0, 1, 3, 0, 3, 2]);
    Ok(backgrounds)
}

pub fn collect_meter_backgrounds(world: &hecs::World) -> Vec<MeterBackgroundData> {
    world
        .query::<(Entity, (&Position, &MeterSize))>()
        .iter()
        .map(|(_, (position, size))| {
            let frame = Frame::new(position.position, size);
            MeterBackgroundData {
                position: frame.bottom_left - Vec3::Y * METER_BACKGROUND_DROP,
                scale: Vec3::new(frame.height, 1.0, frame.width),
                _padding: Default::default(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_spike_detector, spawn_voltmeter, Element};
    use crate::measurement::voltmeter::Corner;
    use crate::rendering::colors::Theme;
    use glam::Vec3Swizzles;
    use hecs::World;

    #[test]
    fn every_meter_gets_a_background_just_under_its_frame() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::new(0.0, 0.0, 10.0));
        let detector = spawn_spike_detector(&mut world, neuron, Vec3::new(-10.0, 0.0, 0.0));
        let backgrounds = collect_meter_backgrounds(&world);
        assert_eq!(backgrounds.len(), 2);
        for meter in [voltmeter, detector] {
            let frame = Frame::of(&world, meter).unwrap();
            let [low, high] =
                [Corner::BottomLeft, Corner::TopRight].map(|corner| frame.corner(corner));
            assert!(backgrounds.iter().any(|background| {
                let far = background.position + background.scale.with_y(0.0);
                background.position.xz() == low.xz()
                    && far.xz() == high.xz()
                    && (-0.2..0.0).contains(&(background.position.y - low.y))
            }));
        }
    }

    #[test]
    fn meter_backgrounds_are_lighter_than_a_light_floor_and_darker_than_a_dark_one() {
        let brightness = |color: Vec3| color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let light = Theme::Light.palette();
        let dark = Theme::Dark.palette();
        assert!(brightness(light.meter_background) > brightness(light.floor));
        assert!(brightness(dark.meter_background) < brightness(dark.floor));
    }
}
