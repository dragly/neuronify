use glam::{Quat, Vec3};
use hecs::{Component, World};

use crate::components::{NeuronType, Position};
use crate::constants::NODE_RADIUS;
use crate::measurement::frame::{Corner, Frame};
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::{ConnectionData, Sphere, TorusData};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{
    generator_color, GENERATOR_RING_RADIUS, GENERATOR_RING_THICKNESS, PAD_RADIUS,
};
use crate::stimuli::poisson_generator::PoissonGenerator;
use crate::stimuli::regular_spike_generator::RegularSpikeGenerator;
use crate::stimuli::visual_input::{grid, RESOLUTION};
use crate::stimuli::{
    alternating_current, current_source, receptive_field, touch_sensor, visual_input,
    GeneratorDynamics, Stimulus,
};

pub const TILE_HEIGHT: f32 = 0.05;
pub const DARK: f32 = 0.04;
pub const FRAME_STRENGTH: f32 = 0.3;
pub const FRAME_LIFT: f32 = 2.0 * TILE_HEIGHT;

impl Stimulus {
    pub fn radius(self) -> f32 {
        match self {
            Stimulus::CurrentSource
            | Stimulus::AlternatingCurrentSource
            | Stimulus::TouchSensor => PAD_RADIUS,
            Stimulus::RegularSpikeGenerator
            | Stimulus::PoissonGenerator
            | Stimulus::ReceptiveField => GENERATOR_RING_RADIUS + GENERATOR_RING_THICKNESS,
            Stimulus::VisualInput => NODE_RADIUS,
        }
    }
}

pub fn shapes(world: &World, palette: &Palette, scene: &mut Scene) {
    for stimulus in Stimulus::ALL {
        match stimulus {
            Stimulus::CurrentSource => current_source::drawing::shapes(world, scene),
            Stimulus::AlternatingCurrentSource => {
                alternating_current::drawing::shapes(world, scene)
            }
            Stimulus::TouchSensor => touch_sensor::drawing::shapes(world, palette, scene),
            Stimulus::RegularSpikeGenerator => {
                generators::<RegularSpikeGenerator>(world, palette, scene)
            }
            Stimulus::PoissonGenerator => generators::<PoissonGenerator>(world, palette, scene),
            Stimulus::VisualInput => visual_input::drawing::shapes(world, scene),
            Stimulus::ReceptiveField => receptive_field::drawing::shapes(world, palette, scene),
        }
    }
}

pub fn lines(world: &World, palette: &Palette) -> Vec<ConnectionData> {
    Stimulus::ALL
        .into_iter()
        .flat_map(|stimulus| match stimulus {
            Stimulus::VisualInput => visual_input::drawing::lines(world),
            Stimulus::ReceptiveField => receptive_field::drawing::lines(world, palette),
            Stimulus::CurrentSource
            | Stimulus::AlternatingCurrentSource
            | Stimulus::TouchSensor
            | Stimulus::RegularSpikeGenerator
            | Stimulus::PoissonGenerator => Vec::new(),
        })
        .collect()
}

fn generators<T: Component>(world: &World, palette: &Palette, scene: &mut Scene) {
    for (position, dynamics, neuron_type) in world
        .query::<(&Position, &GeneratorDynamics, Option<&NeuronType>)>()
        .with::<&T>()
        .iter()
    {
        let color = generator_color(dynamics, neuron_type.copied().unwrap_or_default(), palette);
        scene.spheres.push(Sphere {
            position: position.position,
            color,
            radius: NODE_RADIUS,
            _padding: Default::default(),
        });
        scene.rings.push(TorusData {
            position: position.position,
            major_radius: GENERATOR_RING_RADIUS,
            rotation: Quat::IDENTITY,
            color,
            minor_radius: GENERATOR_RING_THICKNESS,
        });
    }
}

pub fn line(position_a: Vec3, position_b: Vec3, strength: f32, color: Vec3) -> ConnectionData {
    ConnectionData {
        position_a,
        position_b,
        strength,
        directional: 0.0,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    }
}

pub fn tile_in(center: Vec3, size: f32, column: usize, row: usize) -> Vec3 {
    center + Vec3::Z * grid(column) as f32 * size - Vec3::X * grid(row) as f32 * size
        + Vec3::Y * TILE_HEIGHT
}

pub fn tile_at(center: Vec3, size: f32, index: usize) -> Vec3 {
    tile_in(center, size, index % RESOLUTION, index / RESOLUTION)
}

pub fn frame_lines(frame: &Frame, color: Vec3, lines: &mut Vec<ConnectionData>) {
    let corners = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomRight,
        Corner::BottomLeft,
    ]
    .map(|corner| frame.corner(corner) + Vec3::Y * FRAME_LIFT);
    for (index, &corner) in corners.iter().enumerate() {
        lines.push(line(
            corner,
            corners[(index + 1) % corners.len()],
            FRAME_STRENGTH,
            color,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::colors::Theme;

    #[test]
    fn generators_have_rings_and_touch_sensors_are_pads() {
        let mut world = World::new();
        Stimulus::RegularSpikeGenerator.spawn(&mut world, Vec3::ZERO);
        Stimulus::TouchSensor.spawn(&mut world, Vec3::X);
        let palette = Theme::Light.palette();
        let mut scene = Scene::default();
        shapes(&world, &palette, &mut scene);
        assert_eq!(scene.rings.len(), 1);
        let mut pads = Scene::default();
        touch_sensor::drawing::shapes(&world, &palette, &mut pads);
        let pad = pads.cylinders[0];
        assert!((pad.end - pad.start).cross(Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn inhibitory_generators_have_the_inhibitory_colour() {
        let mut world = World::new();
        let generator = Stimulus::PoissonGenerator.spawn(&mut world, Vec3::ZERO);
        *world.get::<&mut NeuronType>(generator).unwrap() = NeuronType::Inhibitory;
        let palette = Theme::Light.palette();
        let mut scene = Scene::default();
        shapes(&world, &palette, &mut scene);
        assert_eq!(scene.rings[0].color, palette.inhibitory);
    }
}
