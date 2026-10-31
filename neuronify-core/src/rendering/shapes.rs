//! Shapes that tell the element types apart without labels.
//!
//! A touch sensor is a flat pad with a pointing hand, a current source is a pad with an
//! arrow towards what it drives, and a spike generator is a soma inside a ring.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_6, PI};

use glam::{Quat, Vec2, Vec3};
use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::visual_input::VisualInput;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{CylinderData, TorusData};
use crate::rendering::scene::Scene;

/// How long a soma or generator keeps glowing after it fires, in simulated seconds.
const FLASH_DURATION: f64 = 0.01;
/// Up and right on the screen with the default camera.
const SCREEN_UP: Vec3 = Vec3::X;
const SCREEN_RIGHT: Vec3 = Vec3::Z;
/// Where the arrow of a current source without a target points.
const UNCONNECTED_ARROW: Vec3 = SCREEN_UP;
pub const PAD_RADIUS: f32 = 1.25 * NODE_RADIUS;
pub const GENERATOR_RING_RADIUS: f32 = 1.6 * NODE_RADIUS;
pub const GENERATOR_RING_THICKNESS: f32 = 0.15 * NODE_RADIUS;
const PAD_HALF_HEIGHT: f32 = 0.2;
const GLYPH_HEIGHT: f32 = PAD_HALF_HEIGHT + 0.04;

struct Digit {
    width: f32,
    side: f32,
    valley: f32,
}

/// The index finger and the knuckles of the hand on a touch pad, after Neuronify 1's icon.
const HAND: [Digit; 4] = [
    Digit {
        width: 1.0,
        side: 2.55,
        valley: 0.45,
    },
    Digit {
        width: 0.55,
        side: 1.36,
        valley: 0.5,
    },
    Digit {
        width: 0.5,
        side: 1.32,
        valley: 0.56,
    },
    Digit {
        width: 0.45,
        side: 1.18,
        valley: 0.6,
    },
];
const HAND_SCALE: f32 = 0.45 * NODE_RADIUS;
const HAND_TILT: f32 = FRAC_PI_6;
const FINGERTIP_STEPS: usize = 8;
const WAVE_STEPS: usize = 16;

/// How much brighter than white a flash starts, so that only flashes bloom.
const FLASH_BRIGHTNESS: f32 = 3.0;

fn flash(time_since_fire: f64) -> f32 {
    let remaining = (1.0 - time_since_fire / FLASH_DURATION).max(0.0) as f32;
    remaining * remaining
}

pub fn voltage_fraction(
    neuron: &LeakyNeuron,
    dynamics: &LeakyDynamics,
    display: &NeuronDisplay,
) -> f32 {
    if !display.voltage {
        return 0.0;
    }
    ((dynamics.voltage - neuron.resting_potential) / (neuron.threshold - neuron.resting_potential))
        .clamp(0.0, 1.0) as f32
}

pub fn shown_voltage(compartment: &Compartment, display: &NeuronDisplay) -> f64 {
    if display.voltage {
        compartment.voltage
    } else {
        Compartment::default().voltage
    }
}

/// The colour of a soma, from its voltage and how recently it fired.
pub fn soma_color(
    world: &hecs::World,
    entity: Entity,
    neuron: &LeakyNeuron,
    dynamics: &LeakyDynamics,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Vec3 {
    let neuron_type = neuron_type(world, entity);
    let rest = neuron.resting_potential;
    let color = if display.voltage && dynamics.voltage < rest {
        let depth = ((rest - dynamics.voltage) / (rest - neuron.minimum_voltage)).clamp(0.0, 1.0);
        palette.soma_below_rest(&neuron_type, depth as f32)
    } else {
        palette.soma(&neuron_type, voltage_fraction(neuron, dynamics, display))
    };
    color.lerp(
        palette.flash * FLASH_BRIGHTNESS,
        flash(dynamics.time_since_fire),
    )
}

pub fn generator_color(
    dynamics: &GeneratorDynamics,
    neuron_type: NeuronType,
    palette: &Palette,
) -> Vec3 {
    palette.generator(neuron_type).lerp(
        palette.flash * FLASH_BRIGHTNESS,
        flash(dynamics.time_since_fire),
    )
}

fn glyph_color() -> Vec3 {
    srgb(255, 244, 214)
}

/// The outline of the hand, centred on the pad, with x to the right and y up on the screen.
pub fn hand_outline() -> Vec<Vec2> {
    let mut points = vec![Vec2::ZERO];
    let mut left = 0.0;
    for digit in HAND {
        let radius = digit.width / 2.0;
        points.extend((0..=FINGERTIP_STEPS).map(|step| {
            let angle = PI * step as f32 / FINGERTIP_STEPS as f32;
            Vec2::new(
                left + radius - radius * angle.cos(),
                digit.side + radius * angle.sin(),
            )
        }));
        left += digit.width;
        points.push(Vec2::new(left, digit.valley));
    }
    let rotation = Vec2::from_angle(HAND_TILT);
    let points: Vec<Vec2> = points
        .iter()
        .map(|&point| rotation.rotate(point) * HAND_SCALE)
        .collect();
    let min = points.iter().copied().reduce(Vec2::min).unwrap_or_default();
    let max = points.iter().copied().reduce(Vec2::max).unwrap_or_default();
    let center = (min + max) / 2.0;
    points.iter().map(|&point| point - center).collect()
}

fn pad(scene: &mut Scene, center: Vec3, color: Vec3) {
    scene.cylinders.push(CylinderData {
        start: center - Vec3::Y * PAD_HALF_HEIGHT,
        start_radius: PAD_RADIUS,
        end: center + Vec3::Y * PAD_HALF_HEIGHT,
        end_radius: PAD_RADIUS,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    });
}

/// The touch pads with their hands, and the current source pads with their arrows.
pub fn collect_shapes(world: &hecs::World, palette: &Palette) -> Scene {
    let mut shapes = Scene::default();
    let hand = hand_outline();
    for (_, (dynamics, position)) in world
        .query::<(&GeneratorDynamics, &Position)>()
        .with::<&TouchSensor>()
        .iter()
    {
        let center = position.position;
        pad(
            &mut shapes,
            center,
            generator_color(dynamics, NeuronType::Excitatory, palette),
        );
        let outline: Vec<Vec3> = hand
            .iter()
            .map(|point| {
                center + Vec3::Y * GLYPH_HEIGHT + SCREEN_RIGHT * point.x + SCREEN_UP * point.y
            })
            .collect();
        shapes.stroke(&outline, 0.06 * NODE_RADIUS, glyph_color());
    }

    let targets: HashMap<Entity, Vec3> = world
        .query::<&Connection>()
        .iter()
        .filter(|(_, connection)| world.get::<&CurrentClamp>(connection.from).is_ok())
        .filter_map(|(_, connection)| {
            let target = world.get::<&Position>(connection.to).ok()?;
            Some((connection.from, target.position))
        })
        .collect();
    for (_, (position, clamp)) in world
        .query::<(&Position, &CurrentClamp)>()
        .with::<&AlternatingCurrent>()
        .iter()
    {
        let center = position.position;
        pad(&mut shapes, center, current_color(clamp.current_output));
        let wave: Vec<Vec3> = (0..=WAVE_STEPS)
            .map(|step| {
                let u = 2.0 * step as f32 / WAVE_STEPS as f32 - 1.0;
                let v = 0.45 * (u * PI).sin();
                center
                    + Vec3::Y * GLYPH_HEIGHT
                    + (SCREEN_RIGHT * u * 0.62 + SCREEN_UP * v) * 0.85 * NODE_RADIUS
            })
            .collect();
        shapes.stroke(&wave, 0.11 * NODE_RADIUS, glyph_color());
    }
    for (entity, (position, clamp)) in world
        .query::<(&Position, &CurrentClamp)>()
        .without::<&AlternatingCurrent>()
        .iter()
    {
        let center = position.position;
        pad(&mut shapes, center, current_color(clamp.current_output));
        let forward = targets
            .get(&entity)
            .map(|target| (*target - center).with_y(0.0).normalize_or_zero())
            .filter(|direction| *direction != Vec3::ZERO)
            .unwrap_or(UNCONNECTED_ARROW);
        let side = Vec3::Y.cross(forward);
        let point = |u: f32, v: f32| {
            center + Vec3::Y * GLYPH_HEIGHT + (side * u + forward * v) * 0.85 * NODE_RADIUS
        };
        let color = glyph_color();
        let radius = 0.11 * NODE_RADIUS;
        shapes.stroke(&[point(0.0, -0.62), point(0.0, 0.62)], radius, color);
        shapes.stroke(
            &[point(-0.4, 0.2), point(0.0, 0.62), point(0.4, 0.2)],
            radius,
            color,
        );
    }
    collect_loudspeakers(world, palette, &mut shapes);
    shapes
}

pub fn speaker_outline() -> Vec<(f32, f32)> {
    vec![
        (-0.55, -0.2),
        (-0.3, -0.2),
        (0.0, -0.5),
        (0.0, 0.5),
        (-0.3, 0.2),
        (-0.55, 0.2),
    ]
}

pub fn speaker_waves() -> Vec<Vec<(f32, f32)>> {
    [0.25, 0.48]
        .map(|radius| {
            (0..=WAVE_STEPS / 2)
                .map(|step| {
                    let angle = (step as f32 / (WAVE_STEPS / 2) as f32 - 0.5) * 1.6;
                    (0.08 + radius * angle.cos(), radius * angle.sin())
                })
                .collect()
        })
        .to_vec()
}

fn collect_loudspeakers(world: &hecs::World, palette: &Palette, shapes: &mut Scene) {
    for (_, (speaker, position)) in world.query::<(&Loudspeaker, &Position)>().iter() {
        let center = position.position;
        let color = speaker_color().lerp(
            palette.flash * FLASH_BRIGHTNESS,
            flash(speaker.time_since_heard),
        );
        pad(shapes, center, color);
        let point = |&(x, y): &(f32, f32)| {
            center
                + Vec3::Y * GLYPH_HEIGHT
                + (SCREEN_RIGHT * x + SCREEN_UP * y) * 0.85 * NODE_RADIUS
        };
        let mut cone: Vec<Vec3> = speaker_outline().iter().map(point).collect();
        cone.push(cone[0]);
        shapes.stroke(&cone, 0.07 * NODE_RADIUS, glyph_color());
        for wave in speaker_waves() {
            let wave: Vec<Vec3> = wave.iter().map(point).collect();
            shapes.stroke(&wave, 0.07 * NODE_RADIUS, glyph_color());
        }
    }
}

/// The rings around spike generators, which flash with the generator.
pub fn collect_rings(world: &hecs::World, palette: &Palette) -> Vec<TorusData> {
    world
        .query::<(&GeneratorDynamics, &Position, Option<&NeuronType>)>()
        .without::<hecs::Or<&TouchSensor, &VisualInput>>()
        .iter()
        .map(|(_, (dynamics, position, neuron_type))| TorusData {
            position: position.position,
            major_radius: GENERATOR_RING_RADIUS,
            rotation: Quat::IDENTITY,
            color: generator_color(dynamics, neuron_type.copied().unwrap_or_default(), palette),
            minor_radius: GENERATOR_RING_THICKNESS,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;
    use crate::simulation::{fire_nearby, lif_step};

    #[test]
    fn a_soma_below_rest_turns_towards_its_own_colder_colour() {
        let mut world = hecs::World::new();
        let palette = Theme::Dark.palette();
        for (element, below) in [
            (Element::ExcitatoryNeuron, palette.excitatory_below),
            (Element::InhibitoryNeuron, palette.inhibitory_below),
        ] {
            let entity = element.spawn(&mut world, Vec3::ZERO);
            let neuron = world.get::<&LeakyNeuron>(entity).unwrap().clone();
            let color_at = |voltage: f64| {
                let dynamics = LeakyDynamics {
                    voltage,
                    ..Default::default()
                };
                soma_color(
                    &world,
                    entity,
                    &neuron,
                    &dynamics,
                    &NeuronDisplay::default(),
                    &palette,
                )
            };
            let rest = color_at(neuron.resting_potential);
            let halfway = color_at((neuron.resting_potential + neuron.minimum_voltage) / 2.0);
            let floor = color_at(neuron.minimum_voltage);
            assert!(floor.abs_diff_eq(below, 1e-6));
            assert!(halfway.distance(below) < rest.distance(below));
            assert!(halfway.abs_diff_eq(rest.lerp(below, 0.5), 1e-6));
        }
    }

    #[test]
    fn generators_have_rings_and_touch_sensors_are_pads() {
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            GeneratorDynamics::default(),
        ));
        world.spawn((
            Position { position: Vec3::X },
            GeneratorDynamics::default(),
            TouchSensor,
        ));
        let palette = Theme::Light.palette();
        assert_eq!(collect_rings(&world, &palette).len(), 1);
        let pad = collect_shapes(&world, &palette).cylinders[0];
        assert!((pad.end - pad.start).cross(Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn inhibitory_generators_have_the_inhibitory_colour() {
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            GeneratorDynamics::default(),
            NeuronType::Inhibitory,
        ));
        let palette = Theme::Light.palette();
        assert_eq!(collect_rings(&world, &palette)[0].color, palette.inhibitory);
    }

    #[test]
    fn touch_pads_carry_a_hand_inside_the_pad() {
        let mut world = hecs::World::new();
        world.spawn((
            Position { position: Vec3::X },
            GeneratorDynamics::default(),
            TouchSensor,
        ));
        let shapes = collect_shapes(&world, &Theme::Dark.palette());
        let (pad, hand) = shapes.cylinders.split_first().unwrap();
        assert!(!hand.is_empty());
        for stroke in hand {
            for end in [stroke.start, stroke.end] {
                assert!(end.y > pad.end.y);
                let reach = (end - Vec3::X).with_y(0.0).length() + stroke.start_radius;
                assert!(reach < PAD_RADIUS, "{reach}");
            }
        }
    }

    #[test]
    fn touch_sensors_stop_flashing_after_a_touch() {
        let mut world = hecs::World::new();
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            GeneratorDynamics::default(),
            TouchSensor,
        ));
        let palette = Theme::Dark.palette();
        let pad_color =
            |world: &hecs::World| collect_shapes(world, &palette).cylinders[0].start_color;
        let resting = pad_color(&world);
        fire_nearby(&mut world, Vec3::ZERO);
        assert_ne!(pad_color(&world), resting);
        let steps = (FLASH_DURATION / LIF_DT).ceil() as usize;
        for step in 0..=steps {
            lif_step(&mut world, LIF_DT, step as f64 * LIF_DT);
        }
        assert_eq!(pad_color(&world), resting);
    }

    #[test]
    fn a_soma_keeps_its_resting_colour_when_the_voltage_is_hidden() {
        let mut world = hecs::World::new();
        let entity = world.spawn((NeuronType::Excitatory,));
        let neuron = LeakyNeuron::default();
        let dynamics = LeakyDynamics {
            voltage: neuron.threshold,
            ..Default::default()
        };
        let palette = Theme::Dark.palette();
        let color = |voltage: bool| {
            let display = NeuronDisplay {
                voltage,
                firing_rate: false,
            };
            soma_color(&world, entity, &neuron, &dynamics, &display, &palette)
        };
        assert_eq!(color(false), palette.excitatory);
        assert_ne!(color(true), palette.excitatory);
    }

    #[test]
    fn alternating_current_sources_carry_a_wave_instead_of_an_arrow() {
        let mut world = hecs::World::new();
        crate::elements::Element::AlternatingCurrentSource.spawn(&mut world, Vec3::ZERO);
        let shapes = collect_shapes(&world, &Theme::Dark.palette());
        let (pad, wave) = shapes.cylinders.split_first().unwrap();
        assert_eq!(wave.len(), WAVE_STEPS);
        let heights: Vec<f32> = wave.iter().map(|segment| segment.start.x).collect();
        assert!(heights.iter().any(|&x| x > 0.2) && heights.iter().any(|&x| x < -0.2));
        assert!(wave.iter().all(|segment| segment.start.y > pad.end.y));
    }

    #[test]
    fn an_alternating_current_source_glows_with_the_size_of_its_current() {
        let pad_brightness = |current: f64| {
            let mut world = hecs::World::new();
            let source =
                crate::elements::Element::AlternatingCurrentSource.spawn(&mut world, Vec3::ZERO);
            world
                .get::<&mut CurrentClamp>(source)
                .unwrap()
                .current_output = current;
            let pad = collect_shapes(&world, &Theme::Dark.palette()).cylinders[0].start_color;
            pad.x + pad.y + pad.z
        };
        assert!(pad_brightness(0.0) < pad_brightness(0.3e-9));
        assert_eq!(pad_brightness(0.3e-9), pad_brightness(-0.3e-9));
    }

    #[test]
    fn a_loudspeaker_flashes_when_it_plays() {
        let mut world = hecs::World::new();
        let neuron = world.spawn((Position { position: Vec3::X },));
        let speaker = crate::elements::spawn_loudspeaker(&mut world, neuron, Vec3::ZERO);
        let palette = Theme::Dark.palette();
        let pad_color =
            |world: &hecs::World| collect_shapes(world, &palette).cylinders[0].start_color;
        assert_eq!(pad_color(&world), speaker_color());
        world
            .get::<&mut Loudspeaker>(speaker)
            .unwrap()
            .time_since_heard = 0.0;
        assert_ne!(pad_color(&world), speaker_color());
    }

    #[test]
    fn current_source_arrow_points_at_its_target() {
        let mut world = hecs::World::new();
        let source = world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            CurrentClamp::default(),
        ));
        let target = world.spawn((Position {
            position: Vec3::new(0.0, 0.0, 5.0),
        },));
        world.spawn((Connection {
            from: source,
            to: target,
            strength: 1.0,
            directional: true,
        },));
        let shapes = collect_shapes(&world, &Theme::Dark.palette());
        let shaft = &shapes.cylinders[1];
        assert!((shaft.end - shaft.start).normalize().dot(Vec3::Z) > 0.99);
    }

    #[test]
    fn a_stronger_current_source_has_a_brighter_pad() {
        let pad_brightness = |current: f64| {
            let mut world = hecs::World::new();
            world.spawn((
                Position {
                    position: Vec3::ZERO,
                },
                CurrentClamp {
                    current_output: current,
                },
            ));
            let pad = collect_shapes(&world, &Theme::Dark.palette()).cylinders[0].start_color;
            pad.x + pad.y + pad.z
        };
        assert!(pad_brightness(0.0) < pad_brightness(0.35e-9));
        assert!(pad_brightness(0.35e-9) < pad_brightness(0.7e-9));
        assert!(pad_brightness(0.7e-9) < pad_brightness(2e-9));
    }
}
