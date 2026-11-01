use std::collections::BTreeMap;

use hecs::{Entity, World};

use crate::components::*;
use crate::constants::BRIDGE_VOLTAGE_THRESHOLD;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::HaloData;
use crate::rendering::paths::ConnectionPath;

const REACH_PER_TERMINAL_RADIUS: f32 = 5.0;
const FULL_RELEASE_EXCESS: f64 = 100.0;
const FAINTEST_RELEASE: f32 = 0.01;

fn synapse_release(synapse: &CurrentSynapse) -> f64 {
    if synapse.alpha_function {
        synapse.linear * synapse.exponential
    } else {
        synapse.exponential
    }
}

fn release(world: &World, entity: Entity, connection: &Connection) -> f32 {
    if let Ok(synapse) = world.get::<&CurrentSynapse>(entity) {
        return synapse_release(&synapse).clamp(0.0, 1.0) as f32;
    }
    if !world
        .satisfies::<&CompartmentCurrent>(entity)
        .unwrap_or(false)
    {
        return 0.0;
    }
    world
        .get::<&Compartment>(connection.from)
        .map(|terminal| {
            let excess = (terminal.voltage - BRIDGE_VOLTAGE_THRESHOLD) / FULL_RELEASE_EXCESS;
            (excess.clamp(0.0, 1.0) * connection.strength.clamp(0.0, 1.0)) as f32
        })
        .unwrap_or(0.0)
}

pub fn collect_transmitter(
    world: &World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
) -> Vec<HaloData> {
    paths
        .iter()
        .filter(|(_, path)| path.directional)
        .filter_map(|(&entity, path)| {
            let connection = world.get::<&Connection>(entity).ok()?;
            let release = release(world, entity, &connection);
            if release < FAINTEST_RELEASE {
                return None;
            }
            let terminal = *path.points.last()?;
            let target = world.get::<&Position>(connection.to).ok()?.position;
            let color = palette.soma(&neuron_type(world, connection.from), 0.0);
            Some(HaloData {
                position: terminal.lerp(target, 0.5),
                radius: REACH_PER_TERMINAL_RADIUS * path.terminal_radius,
                color: color.extend(palette.transmitter * release),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;
    use crate::elements::{spawn_cable, spawn_compartment, spawn_synapse, Element};
    use crate::rendering::cylinders::compartment_radii;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::Theme;
    use crate::simulation::step_simulation;
    use glam::Vec3;

    fn transmitter(world: &World) -> Vec<HaloData> {
        let palette = Theme::Dark.palette();
        let radii = compartment_radii(world);
        collect_transmitter(world, &connection_paths(world, &radii, &palette), &palette)
    }

    fn fire(world: &mut World, neuron: Entity) {
        world.get::<&mut LeakyDynamics>(neuron).unwrap().voltage = 0.0;
    }

    fn strongest_release(world: &mut World, steps: usize) -> Option<HaloData> {
        let mut time = 0.0;
        let mut strongest: Option<HaloData> = None;
        for _ in 0..steps {
            step_simulation(world, &mut time, 1, None);
            for cloud in transmitter(world) {
                if strongest.is_none_or(|strongest| cloud.color.w > strongest.color.w) {
                    strongest = Some(cloud);
                }
            }
        }
        strongest
    }

    fn synapse_from(element: Element) -> (World, Entity, Entity) {
        let mut world = World::new();
        let source = element.spawn(&mut world, Vec3::ZERO);
        let target = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(10.0, 0.0, 0.0));
        spawn_synapse(&mut world, source, target);
        (world, source, target)
    }

    #[test]
    fn a_quiet_synapse_releases_nothing() {
        let (mut world, _, _) = synapse_from(Element::ExcitatoryNeuron);
        assert!(strongest_release(&mut world, 200).is_none());
    }

    #[test]
    fn transmitter_is_released_between_the_terminal_and_its_target() {
        let (mut world, source, _) = synapse_from(Element::ExcitatoryNeuron);
        fire(&mut world, source);
        let cloud = strongest_release(&mut world, 200).unwrap();
        let from_target = cloud.position.distance(Vec3::new(10.0, 0.0, 0.0));
        assert!((0.5 * NODE_RADIUS..2.0 * NODE_RADIUS).contains(&from_target));
        assert!(cloud.radius > 2.0 * NODE_RADIUS);
        let palette = Theme::Dark.palette();
        assert_eq!(cloud.color.w, palette.transmitter);
        assert_eq!(cloud.color.truncate(), palette.excitatory);
    }

    #[test]
    fn inhibitory_transmitter_has_the_inhibitory_color() {
        let (mut world, source, _) = synapse_from(Element::InhibitoryNeuron);
        fire(&mut world, source);
        let cloud = strongest_release(&mut world, 200).unwrap();
        assert_eq!(cloud.color.truncate(), Theme::Dark.palette().inhibitory);
    }

    #[test]
    fn released_transmitter_fades_as_the_synapse_closes() {
        let (mut world, source, _) = synapse_from(Element::InhibitoryNeuron);
        fire(&mut world, source);
        let mut time = 0.0;
        let mut opacities = Vec::new();
        for _ in 0..400 {
            step_simulation(&mut world, &mut time, 1, None);
            opacities.extend(transmitter(&world).iter().map(|cloud| cloud.color.w));
        }
        let peak = opacities
            .iter()
            .position(|&opacity| opacity == Theme::Dark.palette().transmitter)
            .unwrap();
        assert!(opacities[peak..].windows(2).all(|pair| pair[1] < pair[0]));
        assert!(transmitter(&world).is_empty());
    }

    #[test]
    fn an_alpha_synapse_releases_at_most_fully() {
        let (mut world, source, _) = synapse_from(Element::ExcitatoryNeuron);
        for (_, synapse) in world.query_mut::<&mut CurrentSynapse>() {
            synapse.alpha_function = true;
        }
        fire(&mut world, source);
        let cloud = strongest_release(&mut world, 200).unwrap();
        let full = Theme::Dark.palette().transmitter;
        assert!(cloud.color.w > 0.95 * full && cloud.color.w <= full);
    }

    fn inhibitory_axon_ending(strength: f64) -> (World, Entity) {
        let mut world = World::new();
        let source = Element::InhibitoryNeuron.spawn(&mut world, Vec3::ZERO);
        let target = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(10.0, 0.0, 0.0));
        let mut previous = source;
        for step in 1..4 {
            let position = Vec3::new(2.0 * step as f32, 0.0, 0.0);
            let compartment = spawn_compartment(&mut world, position, NeuronType::Inhibitory);
            spawn_cable(&mut world, previous, compartment, false);
            previous = compartment;
        }
        let ending = spawn_cable(&mut world, previous, target, true);
        world.get::<&mut Connection>(ending).unwrap().strength = strength;
        (world, source)
    }

    #[test]
    fn a_weaker_axon_ending_releases_fainter_transmitter() {
        let strongest = |strength: f64| {
            let (mut world, source) = inhibitory_axon_ending(strength);
            fire(&mut world, source);
            strongest_release(&mut world, 2000).map(|cloud| cloud.color.w)
        };
        let full = strongest(1.0).unwrap();
        assert_eq!(strongest(2.0), Some(full));
        assert!((strongest(0.5).unwrap() - 0.5 * full).abs() < 1e-6);
        assert_eq!(strongest(0.0), None);
    }

    #[test]
    fn an_axon_terminal_releases_transmitter_when_its_spike_arrives() {
        let (mut world, source) = inhibitory_axon_ending(1.0);
        assert!(strongest_release(&mut world, 200).is_none());

        fire(&mut world, source);
        let cloud = strongest_release(&mut world, 2000).unwrap();
        let palette = Theme::Dark.palette();
        assert!(cloud.color.w > 0.8 * palette.transmitter);
        assert_eq!(cloud.color.truncate(), palette.inhibitory);
        let from_target = cloud.position.distance(Vec3::new(10.0, 0.0, 0.0));
        assert!((0.5 * NODE_RADIUS..2.0 * NODE_RADIUS).contains(&from_target));
    }
}
