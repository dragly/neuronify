use glam::{Quat, Vec3};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::{COMPARTMENT_SPHERE_SCALE, ERASE_RADIUS, NODE_RADIUS};
use crate::measurement::voltmeter::{Corner, Frame};
use crate::rendering::bounds::note_corners;
use crate::rendering::colors::Palette;
use crate::rendering::cylinders::compartment_radii;
use crate::rendering::gauges::gauge_reach;
use crate::rendering::gpu_types::{Sphere, TorusData};
use crate::rendering::scene::Scene;
use crate::rendering::shapes::{GENERATOR_RING_RADIUS, GENERATOR_RING_THICKNESS, PAD_RADIUS};

const GAP: f32 = 0.35 * NODE_RADIUS;
const THICKNESS: f32 = 0.07 * NODE_RADIUS;
const HANDLE_RADIUS: f32 = 0.3 * NODE_RADIUS;

pub fn collect_selection(world: &World, selection: Entity, palette: &Palette) -> Scene {
    let mut scene = Scene::default();
    if let Some([min, max]) = rectangle(world, selection) {
        let min = min - Vec3::new(GAP, 0.0, GAP);
        let max = max + Vec3::new(GAP, 0.0, GAP);
        let outline = [
            min,
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, min.y, max.z),
            min,
        ];
        scene.stroke(&outline, THICKNESS, palette.selection);
        scene
            .spheres
            .extend(outline[..4].iter().map(|&corner| Sphere {
                position: corner,
                color: palette.selection,
                radius: HANDLE_RADIUS,
                _padding: Default::default(),
            }));
    } else if let Ok(position) = world.get::<&Position>(selection) {
        scene.rings.push(TorusData {
            position: position.position,
            major_radius: reach(world, selection) + GAP,
            rotation: Quat::IDENTITY,
            color: palette.selection,
            minor_radius: THICKNESS,
        });
    }
    scene
}

pub fn eraser_outline(center: Vec3, palette: &Palette) -> TorusData {
    TorusData {
        position: center,
        major_radius: ERASE_RADIUS,
        rotation: Quat::IDENTITY,
        color: palette.neutral,
        minor_radius: THICKNESS,
    }
}

fn rectangle(world: &World, entity: Entity) -> Option<[Vec3; 2]> {
    if let Some(frame) = Frame::of(world, entity) {
        return Some([
            frame.corner(Corner::BottomLeft),
            frame.corner(Corner::TopRight),
        ]);
    }
    let position = world.get::<&Position>(entity).ok()?;
    let size = world.get::<&NoteSize>(entity).ok()?;
    Some(note_corners(position.position, &size))
}

fn reach(world: &World, entity: Entity) -> f32 {
    if world.get::<&TouchSensor>(entity).is_ok() || world.get::<&CurrentClamp>(entity).is_ok() {
        PAD_RADIUS
    } else if world.get::<&GeneratorDynamics>(entity).is_ok() {
        GENERATOR_RING_RADIUS + GENERATOR_RING_THICKNESS
    } else if world.get::<&Compartment>(entity).is_ok() {
        compartment_radii(world)
            .get(&entity)
            .copied()
            .unwrap_or(COMPARTMENT_SPHERE_SCALE * NODE_RADIUS)
    } else {
        gauge_reach(world, entity).max(NODE_RADIUS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_compartment, spawn_voltmeter, Element};
    use crate::rendering::bounds::circuit_bounds;
    use crate::rendering::colors::Theme;

    fn select(world: &World, entity: Entity) -> Scene {
        collect_selection(world, entity, &Theme::Dark.palette())
    }

    fn assert_ring_clears(world: &World, entity: Entity, what: &str) {
        let selection = select(world, entity);
        assert!(selection.cylinders.is_empty(), "{what}");
        assert!(selection.spheres.is_empty(), "{what}");
        let [ring] = selection.rings[..] else {
            panic!("{what} has no ring");
        };
        let center = world.get::<&Position>(entity).unwrap().position;
        let bounds = circuit_bounds(world).unwrap();
        let drawn = (bounds.max - center)
            .max(center - bounds.min)
            .max_element()
            .max(gauge_reach(world, entity));
        assert_eq!(ring.position, center, "{what}");
        assert!(ring.major_radius - ring.minor_radius > drawn, "{what}");
    }

    #[test]
    fn round_elements_get_a_ring_around_all_they_draw() {
        for element in [
            Element::ExcitatoryNeuron,
            Element::InhibitoryNeuron,
            Element::AdaptiveNeuron,
            Element::BurstNeuron,
            Element::CurrentSource,
            Element::TouchSensor,
            Element::RegularSpikeGenerator,
            Element::PoissonGenerator,
        ] {
            let mut world = World::new();
            let entity = element.spawn(&mut world, Vec3::new(3.0, 0.0, 4.0));
            assert_ring_clears(&world, entity, &format!("{element:?}"));
        }
        let mut world = World::new();
        let compartment = spawn_compartment(&mut world, Vec3::X, NeuronType::Excitatory);
        assert_ring_clears(&world, compartment, "a compartment");
    }

    #[test]
    fn voltmeters_and_notes_get_a_rectangle_with_handles() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::new(10.0, 0.0, 5.0));
        let note = Element::Note.spawn(&mut world, Vec3::new(-10.0, 0.0, 0.0));
        for entity in [voltmeter, note] {
            let [min, max] = rectangle(&world, entity).unwrap();
            let selection = select(&world, entity);
            assert!(selection.rings.is_empty());
            assert_eq!(selection.cylinders.len(), 4);
            let ends: Vec<Vec3> = selection
                .cylinders
                .iter()
                .flat_map(|cylinder| [cylinder.start, cylinder.end])
                .collect();
            let outline_min = ends.iter().copied().reduce(Vec3::min).unwrap();
            let outline_max = ends.iter().copied().reduce(Vec3::max).unwrap();
            assert!(outline_min.x < min.x && outline_min.z < min.z);
            assert!(outline_max.x > max.x && outline_max.z > max.z);
            let handles: Vec<Vec3> = selection
                .spheres
                .iter()
                .filter(|sphere| sphere.radius == HANDLE_RADIUS)
                .map(|sphere| sphere.position)
                .collect();
            assert_eq!(handles.len(), 4);
            for handle in handles {
                assert!(handle.x == outline_min.x || handle.x == outline_max.x);
                assert!(handle.z == outline_min.z || handle.z == outline_max.z);
            }
        }
    }

    #[test]
    fn an_erased_selection_draws_nothing() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        world.despawn(neuron).unwrap();
        let selection = select(&world, neuron);
        assert!(selection.rings.is_empty() && selection.cylinders.is_empty());
    }
}
