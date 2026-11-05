use std::collections::{BTreeMap, HashMap};
use std::f32::consts::FRAC_PI_4;

use glam::{Quat, Vec3};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::{FLOOR_HEIGHT, NODE_RADIUS};
use crate::measurement::is_meter;
use crate::rendering::colors::{current_color, Palette, Theme};
use crate::rendering::cylinders::compartment_radii;
use crate::rendering::gpu_types::{CylinderData, TorusData};
use crate::rendering::scene::Scene;
use crate::rendering::selection::element_radius;
use crate::rendering::shapes::glyph_color;

const GAP: f32 = 0.3 * NODE_RADIUS;
const SELECTED_THICKNESS: f32 = 0.07 * NODE_RADIUS;
const THINNEST: f32 = 0.1 * NODE_RADIUS;
const THICKNESS_PER_STRENGTH: f32 = 0.2 * NODE_RADIUS;
const THICKEST: f32 = 0.45 * NODE_RADIUS;
const RING_SPACING: f32 = 0.2 * NODE_RADIUS;
const BADGE_RADIUS: f32 = 0.75 * NODE_RADIUS;
const BADGE_SPACING: f32 = 2.0 * BADGE_RADIUS + 0.15 * NODE_RADIUS;
const BADGE_THICKNESS: f32 = 0.15 * NODE_RADIUS;
const GLYPH_LIFT: f32 = 0.04 * NODE_RADIUS;
const ARROW_HALF_LENGTH: f32 = 0.45 * NODE_RADIUS;
const ARROW_HEAD: f32 = 0.27 * NODE_RADIUS;
const ARROW_THICKNESS: f32 = 0.1 * NODE_RADIUS;
const UP: Vec3 = Vec3::X;
const RIGHT: Vec3 = Vec3::Z;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flow {
    Into,
    OutOf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ring {
    thickness: f32,
    color: Vec3,
}

pub fn collect_connection_rings(world: &World, palette: &Palette) -> Scene {
    let mut scene = Scene::default();
    if ConnectionView::of(world) != ConnectionView::Rings {
        return scene;
    }
    let Some(selected) = Selected::entity(world) else {
        return scene;
    };
    let radii = compartment_radii(world);
    if let Ok(position) = world.get::<&Position>(selected) {
        if ringed_selection(world) == Some(selected) {
            scene.rings.push(lying(
                position.position,
                element_radius(world, selected, &radii) + GAP + THICKEST,
                THICKEST,
                palette.selection,
            ));
        } else if !is_meter(world, selected) {
            scene.rings.push(torus(
                position.position,
                element_radius(world, selected, &radii) + GAP,
                SELECTED_THICKNESS,
                palette.selection,
            ));
        }
    }
    for (neighbour, rings) in neighbour_rings(world, selected, palette) {
        let Ok(center) = world.get::<&Position>(neighbour).map(|p| p.position) else {
            continue;
        };
        let (ring_radii, edge) = ring_radii(world, neighbour, &rings, &radii);
        for ((ring, _), radius) in rings.iter().zip(ring_radii) {
            scene
                .rings
                .push(lying(center, radius, ring.thickness, ring.color));
        }
        let height = rings
            .iter()
            .map(|(ring, _)| 2.0 * ring.thickness)
            .fold(0.0, f32::max);
        let separation = 2.0 * (0.5 * BADGE_SPACING / edge).min(1.0).asin();
        let middle = (rings.len() as f32 - 1.0) / 2.0;
        let labelled = world
            .get::<&Label>(neighbour)
            .is_ok_and(|label| !label.text.is_empty());
        let corner = if labelled { -FRAC_PI_4 } else { FRAC_PI_4 };
        for (index, (ring, flows)) in rings.iter().enumerate() {
            let angle = corner + (middle - index as f32) * separation;
            badge(
                &mut scene,
                on_floor(center + (UP * angle.sin() + RIGHT * angle.cos()) * edge)
                    + Vec3::Y * height,
                flows,
                ring.color,
            );
        }
    }
    scene
}

pub fn ringed_selection(world: &World) -> Option<Entity> {
    Selected::entity(world).filter(|&selected| {
        ConnectionView::of(world) == ConnectionView::Rings && is_neuron(world, selected)
    })
}

pub fn ring_edges(world: &World) -> BTreeMap<Entity, f32> {
    let Some(selected) = Selected::entity(world) else {
        return BTreeMap::new();
    };
    let radii = compartment_radii(world);
    neighbour_rings(world, selected, &Theme::default().palette())
        .into_iter()
        .map(|(neighbour, rings)| (neighbour, ring_radii(world, neighbour, &rings, &radii).1))
        .collect()
}

fn ring_radii(
    world: &World,
    neighbour: Entity,
    rings: &[(Ring, Vec<Flow>)],
    compartment_radii: &HashMap<Entity, f32>,
) -> (Vec<f32>, f32) {
    let mut inner = element_radius(world, neighbour, compartment_radii) + GAP;
    let mut radii = Vec::new();
    for (ring, _) in rings {
        radii.push(inner + ring.thickness);
        inner += 2.0 * ring.thickness + RING_SPACING;
    }
    (radii, inner - RING_SPACING)
}

fn neighbour_rings(
    world: &World,
    selected: Entity,
    palette: &Palette,
) -> BTreeMap<Entity, Vec<(Ring, Vec<Flow>)>> {
    let mut neighbours: BTreeMap<Entity, [Option<Ring>; 2]> = BTreeMap::new();
    for (connection, neighbour) in ringed_connections(world) {
        let flow = usize::from(connection.to == selected);
        let ring = Ring {
            thickness: (THINNEST + THICKNESS_PER_STRENGTH * connection.strength as f32)
                .min(THICKEST),
            color: sender_color(world, connection.from, palette),
        };
        let slot = &mut neighbours.entry(neighbour).or_default()[flow];
        if slot.is_none_or(|existing| existing.thickness < ring.thickness) {
            *slot = Some(ring);
        }
    }
    neighbours
        .into_iter()
        .map(|(neighbour, [into, out_of])| {
            let rings = match (into, out_of) {
                (Some(into), Some(out_of)) if into.color == out_of.color => vec![(
                    Ring {
                        thickness: into.thickness.max(out_of.thickness),
                        color: into.color,
                    },
                    vec![Flow::Into, Flow::OutOf],
                )],
                _ => [(into, Flow::Into), (out_of, Flow::OutOf)]
                    .into_iter()
                    .filter_map(|(ring, flow)| ring.map(|ring| (ring, vec![flow])))
                    .collect(),
            };
            (neighbour, rings)
        })
        .collect()
}

fn torus(position: Vec3, major_radius: f32, minor_radius: f32, color: Vec3) -> TorusData {
    TorusData {
        position,
        major_radius,
        rotation: Quat::IDENTITY,
        color,
        minor_radius,
    }
}

fn on_floor(point: Vec3) -> Vec3 {
    Vec3::new(point.x, FLOOR_HEIGHT, point.z)
}

fn lying(center: Vec3, major_radius: f32, thickness: f32, color: Vec3) -> TorusData {
    torus(
        on_floor(center) + Vec3::Y * thickness,
        major_radius,
        thickness,
        color,
    )
}

fn badge(scene: &mut Scene, base: Vec3, flows: &[Flow], color: Vec3) {
    scene.cylinders.push(CylinderData {
        start: base,
        start_radius: BADGE_RADIUS,
        end: base + Vec3::Y * BADGE_THICKNESS,
        end_radius: BADGE_RADIUS,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    });
    let glyph = base + Vec3::Y * (BADGE_THICKNESS + GLYPH_LIFT);
    let left = glyph - RIGHT * ARROW_HALF_LENGTH;
    let right = glyph + RIGHT * ARROW_HALF_LENGTH;
    scene.stroke(&[left, right], ARROW_THICKNESS, glyph_color());
    for flow in flows {
        let (tip, back) = match flow {
            Flow::Into => (left, RIGHT),
            Flow::OutOf => (right, -RIGHT),
        };
        scene.stroke(
            &[
                tip + (back + UP) * ARROW_HEAD,
                tip,
                tip + (back - UP) * ARROW_HEAD,
            ],
            ARROW_THICKNESS,
            glyph_color(),
        );
    }
}

fn sender_color(world: &World, sender: Entity, palette: &Palette) -> Vec3 {
    let Ok(sender) = world.entity(sender) else {
        return palette.neutral;
    };
    if let Some(clamp) = sender.get::<&CurrentClamp>() {
        current_color(clamp.current_output)
    } else if sender.has::<GeneratorDynamics>() {
        palette.generator(
            sender
                .get::<&NeuronType>()
                .map_or_else(Default::default, |neuron_type| *neuron_type),
        )
    } else if let Some(neuron_type) = sender.get::<&NeuronType>() {
        match *neuron_type {
            NeuronType::Excitatory => palette.excitatory,
            NeuronType::Inhibitory => palette.inhibitory,
        }
    } else {
        palette.neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_spike_detector, spawn_synapse, Element};
    use crate::rendering::colors::Theme;
    use crate::rendering::connections::collect_meter_wires;
    use crate::rendering::selection::collect_selection;

    struct Circuit {
        world: World,
        cell: Entity,
        excited: Entity,
        partner: Entity,
    }

    fn circuit() -> Circuit {
        let mut world = World::new();
        let [cell, excited] =
            [0.0, 6.0].map(|z| Element::ExcitatoryNeuron.spawn(&mut world, Vec3::Z * z));
        let partner = Element::InhibitoryNeuron.spawn(&mut world, Vec3::X * -6.0);
        spawn_synapse(&mut world, cell, excited);
        spawn_synapse(&mut world, partner, cell);
        ConnectionView::Rings.set_in(&mut world);
        Selected::set(&mut world, Some(cell));
        Circuit {
            world,
            cell,
            excited,
            partner,
        }
    }

    fn rings(world: &World, selected: Entity) -> BTreeMap<Entity, Vec<(Vec3, Vec<Flow>)>> {
        neighbour_rings(world, selected, &Theme::Dark.palette())
            .into_iter()
            .map(|(neighbour, rings)| {
                (
                    neighbour,
                    rings
                        .into_iter()
                        .map(|(ring, flows)| (ring.color, flows))
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn rings_replace_the_lines_of_the_selected_neuron_and_say_which_way_each_goes() {
        let Circuit {
            world,
            cell,
            excited,
            partner,
        } = circuit();
        let palette = Theme::Dark.palette();
        assert_eq!(hidden_connections(&world).len(), 2);
        let rings = rings(&world, cell);
        assert_eq!(rings[&excited], [(palette.excitatory, vec![Flow::Into])]);
        assert_eq!(rings[&partner], [(palette.inhibitory, vec![Flow::OutOf])]);
        let scene = collect_connection_rings(&world, &palette);
        assert_eq!(scene.rings.len(), 3);
        let badges = scene
            .cylinders
            .iter()
            .filter(|cylinder| cylinder.start_radius == BADGE_RADIUS);
        assert_eq!(badges.count(), 2);
        assert!(scene
            .rings
            .iter()
            .any(|ring| ring.color == palette.selection));
    }

    #[test]
    fn the_rings_lie_on_the_floor_and_the_selected_neuron_gets_the_thickest() {
        let Circuit { world, cell, .. } = circuit();
        let palette = Theme::Dark.palette();
        let scene = collect_connection_rings(&world, &palette);
        let height = world.get::<&Position>(cell).unwrap().position.y;
        for ring in &scene.rings {
            assert!((ring.position.y - ring.minor_radius - FLOOR_HEIGHT).abs() < 1e-6);
            assert!(ring.position.y < height);
        }
        let selection = scene
            .rings
            .iter()
            .find(|ring| ring.color == palette.selection)
            .unwrap();
        assert_eq!(selection.minor_radius, THICKEST);
        assert!(collect_selection(&world, cell, &palette).rings.is_empty());
    }

    #[test]
    fn a_badge_sits_at_the_upper_right_unless_a_label_is_there() {
        let Circuit {
            mut world, excited, ..
        } = circuit();
        let badge_up = |world: &World| {
            collect_connection_rings(world, &Theme::Dark.palette())
                .cylinders
                .iter()
                .find(|cylinder| cylinder.start_radius == BADGE_RADIUS && cylinder.end.z > 3.0)
                .map(|badge| badge.end.dot(UP))
                .unwrap()
        };
        assert!(badge_up(&world) > 0.0);
        world
            .insert_one(
                excited,
                Label {
                    text: "Reset".into(),
                },
            )
            .unwrap();
        assert!(badge_up(&world) < 0.0);
    }

    #[test]
    fn an_arrow_points_left_into_a_neuron_and_right_out_of_it() {
        let Circuit {
            world,
            excited,
            partner,
            ..
        } = circuit();
        let scene = collect_connection_rings(&world, &Theme::Dark.palette());
        let leaning = |neuron: Entity| {
            let center = world.get::<&Position>(neuron).unwrap().position;
            let badge = scene
                .cylinders
                .iter()
                .filter(|cylinder| cylinder.start_radius == BADGE_RADIUS)
                .map(|badge| badge.end)
                .min_by(|a, b| a.distance(center).total_cmp(&b.distance(center)))
                .unwrap();
            scene
                .cylinders
                .iter()
                .filter(|cylinder| {
                    cylinder.start_color == glyph_color()
                        && cylinder.start.distance(badge) < BADGE_RADIUS
                })
                .flat_map(|cylinder| [cylinder.start, cylinder.end])
                .map(|point| (point - badge).dot(RIGHT))
                .sum::<f32>()
        };
        assert!(leaning(excited) < 0.0);
        assert!(leaning(partner) > 0.0);
    }

    #[test]
    fn the_direction_setting_keeps_only_inputs_or_only_outputs() {
        let Circuit {
            mut world,
            cell,
            excited,
            partner,
        } = circuit();
        ConnectionDirection::Inputs.set_in(&mut world);
        assert_eq!(rings(&world, cell).keys().collect::<Vec<_>>(), [&partner]);
        ConnectionDirection::Outputs.set_in(&mut world);
        assert_eq!(rings(&world, cell).keys().collect::<Vec<_>>(), [&excited]);
        ConnectionView::LastSelected.set_in(&mut world);
        assert_eq!(hidden_connections(&world).len(), 1);
    }

    #[test]
    fn a_pair_connected_both_ways_shares_a_ring_unless_the_signs_differ() {
        let Circuit {
            mut world,
            cell,
            excited,
            partner,
        } = circuit();
        spawn_synapse(&mut world, excited, cell);
        spawn_synapse(&mut world, cell, partner);
        let palette = Theme::Dark.palette();
        let rings = rings(&world, cell);
        assert_eq!(
            rings[&excited],
            [(palette.excitatory, vec![Flow::Into, Flow::OutOf])]
        );
        assert_eq!(
            rings[&partner],
            [
                (palette.excitatory, vec![Flow::Into]),
                (palette.inhibitory, vec![Flow::OutOf])
            ]
        );
    }

    #[test]
    fn meters_and_sensors_keep_their_wires_and_other_views_draw_no_rings() {
        let Circuit {
            mut world, cell, ..
        } = circuit();
        let detector = spawn_spike_detector(&mut world, cell, Vec3::X * 10.0);
        let wire = world
            .query::<(Entity, &Connection)>()
            .iter()
            .find(|(_, connection)| connection.to == detector)
            .map(|(wire, _)| wire)
            .unwrap();
        let sensor = Element::TouchSensor.spawn(&mut world, Vec3::X * -3.0);
        let touch = spawn_synapse(&mut world, sensor, cell);
        let hidden = hidden_connections(&world);
        assert!(!hidden.contains(&wire) && !hidden.contains(&touch));
        let start = world.get::<&Position>(cell).unwrap().position;
        assert!(collect_meter_wires(&world, &Theme::Dark.palette())
            .iter()
            .any(|segment| segment.start.distance(start) < 1e-3));
        let rings = rings(&world, cell);
        assert!(!rings.contains_key(&detector) && !rings.contains_key(&sensor));
        ConnectionView::All.set_in(&mut world);
        let scene = collect_connection_rings(&world, &Theme::Dark.palette());
        assert!(scene.rings.is_empty() && scene.cylinders.is_empty());
    }
}
