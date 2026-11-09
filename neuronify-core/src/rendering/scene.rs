use std::collections::{BTreeMap, HashMap};

use glam::Vec3;
use hecs::Entity;

use crate::components::{hidden_connections, Milliseconds};
use crate::rendering::colors::Palette;
use crate::rendering::current_flow::collect_current_flow;
use crate::rendering::cylinders::{collect_cylinders, compartment_radii};
use crate::rendering::gpu_types::{CylinderData, HaloData, Sphere, TorusData};
use crate::rendering::paths::{connection_paths, ConnectionPath};
use crate::rendering::shapes::{collect_rings, collect_shapes};
use crate::rendering::signals::collect_signals;
use crate::rendering::spheres::collect_spheres;
use crate::rendering::transmitter::collect_transmitter;
use crate::rendering::visual_input::collect_visual_input_tiles;

/// The shapes of the 3D view. The example previews draw the solid ones from above.
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub cylinders: Vec<CylinderData>,
    pub rings: Vec<TorusData>,
    pub transmitter: Vec<HaloData>,
}

impl Scene {
    pub fn stroke(&mut self, points: &[Vec3], radius: f32, color: Vec3) {
        for pair in points.windows(2) {
            self.cylinders.push(CylinderData {
                start: pair[0],
                start_radius: radius,
                end: pair[1],
                end_radius: radius,
                start_color: color,
                end_color: color,
                _padding: Default::default(),
            });
        }
        for &position in points {
            self.spheres.push(Sphere {
                position,
                color,
                radius,
                _padding: Default::default(),
            });
        }
    }

    pub fn extend(&mut self, other: Scene) {
        self.spheres.extend(other.spheres);
        self.cylinders.extend(other.cylinders);
        self.rings.extend(other.rings);
        self.transmitter.extend(other.transmitter);
    }
}

pub fn collect_scene(world: &hecs::World, palette: &Palette, time: Milliseconds) -> Scene {
    let radii = compartment_radii(world);
    let paths = shown_paths(world, &radii, palette);
    collect_scene_along(world, &radii, &paths, palette, time)
}

pub fn collect_scene_along(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
    time: Milliseconds,
) -> Scene {
    let mut scene = Scene {
        spheres: collect_spheres(world, radii, paths, palette),
        cylinders: collect_cylinders(world, radii, paths, palette),
        rings: collect_rings(world, palette),
        transmitter: Vec::new(),
    };
    scene.extend(collect_transmitter(world, paths, radii, palette, time));
    scene
        .spheres
        .extend(collect_current_flow(world, paths, time));
    scene.extend(collect_signals(world, paths, palette, time));
    scene.extend(collect_shapes(world, palette));
    scene.cylinders.extend(collect_visual_input_tiles(world));
    scene
}

pub fn shown_paths(
    world: &hecs::World,
    radii: &HashMap<Entity, f32>,
    palette: &Palette,
) -> BTreeMap<Entity, ConnectionPath> {
    let hidden = hidden_connections(world);
    let mut paths = connection_paths(world, radii, palette);
    paths.retain(|entity, _| !hidden.contains(entity));
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::elements::{spawn_cable, spawn_compartment};
    use crate::rendering::colors::Theme;
    use hecs::World;

    fn neuron(world: &mut World, z: f32) -> Entity {
        world.spawn((
            Position {
                position: Vec3::new(0.0, 0.0, z),
            },
            LeakyNeuron::default(),
            NodeState::default(),
            NeuronType::Excitatory,
        ))
    }

    fn connect(world: &mut World, from: Entity, to: Entity) {
        world.spawn((Connection {
            from,
            to,
            strength: 1.0,
            directional: true,
        },));
    }

    fn shown(world: &mut World, view: ConnectionView, selected: Option<Entity>) -> usize {
        view.set_in(world);
        Selected::set(world, selected);
        let palette = Theme::default().palette();
        shown_paths(world, &compartment_radii(world), &palette).len()
    }

    #[test]
    fn last_selected_view_shows_only_connections_of_the_last_selected_neuron() {
        let mut world = World::new();
        let [a, b, c] = [0.0, 6.0, 12.0].map(|z| neuron(&mut world, z));
        connect(&mut world, a, b);
        connect(&mut world, b, c);
        assert_eq!(shown(&mut world, ConnectionView::All, None), 2);
        assert_eq!(shown(&mut world, ConnectionView::LastSelected, None), 0);
        assert_eq!(shown(&mut world, ConnectionView::LastSelected, Some(a)), 1);
        assert_eq!(shown(&mut world, ConnectionView::LastSelected, Some(b)), 2);
        assert_eq!(shown(&mut world, ConnectionView::None, Some(b)), 0);
    }

    #[test]
    fn hidden_connections_leave_the_axon_of_a_neuron() {
        let mut world = World::new();
        let [a, b] = [0.0, 6.0].map(|z| neuron(&mut world, z));
        connect(&mut world, a, b);
        let terminal = spawn_compartment(&mut world, Vec3::X * 3.0, NeuronType::Excitatory);
        spawn_cable(&mut world, a, terminal, true);
        assert_eq!(shown(&mut world, ConnectionView::All, None), 2);
        assert_eq!(shown(&mut world, ConnectionView::None, None), 1);
    }
}
