pub mod drawing;
pub mod gui;

use std::collections::HashMap;
use std::ops::RangeInclusive;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Milliseconds};
use serde::{Deserialize, Serialize};

use crate::components::Connection;
use crate::serialization::bounds::{turned, within};
use crate::simulation::engine::Engine;
use crate::stimuli::visual_input::{
    grid, is_picture_wire, VisualInput, FRAME_INTERVAL, RESOLUTION, VISUAL_INPUT_SIZE,
};
use crate::stimuli::About;

pub const RECEPTIVE_FIELD_SIZE: f32 = 4.0;
const FIELD_GAP: f32 = 1.5;
pub const SENSITIVITIES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(1e6);
pub const OFFSETS: RangeInclusive<f64> = -0.5..=0.5;

pub const ABOUT: About = About {
    name: "Receptive field",
    description: "Watches part of a picture and fires when the picture matches it. \
                  Connect a visual input to it.",
    size: Some(RECEPTIVE_FIELD_SIZE),
};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum ReceptiveField {
    Orientation { angle: f64, x: f64, y: f64 },
    CenterSurround { off_center: bool, x: f64, y: f64 },
    Edge { angle: f64 },
}

impl Default for ReceptiveField {
    fn default() -> Self {
        ReceptiveField::Orientation {
            angle: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl ReceptiveField {
    pub const KINDS: [ReceptiveField; 3] = [
        ReceptiveField::Orientation {
            angle: 0.0,
            x: 0.0,
            y: 0.0,
        },
        ReceptiveField::CenterSurround {
            off_center: false,
            x: 0.0,
            y: 0.0,
        },
        ReceptiveField::Edge { angle: 0.0 },
    ];

    pub fn name(self) -> &'static str {
        match self {
            ReceptiveField::Orientation { .. } => "Orientation",
            ReceptiveField::CenterSurround { .. } => "Center",
            ReceptiveField::Edge { .. } => "Edge",
        }
    }

    pub fn same_kind(self, other: ReceptiveField) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }

    pub fn weight(self, x: f64, y: f64) -> f64 {
        match self {
            ReceptiveField::Orientation {
                angle,
                x: x0,
                y: y0,
            } => {
                let (width, length, frequency) = (0.1, 0.2, 20.0);
                let along = (x - x0) * angle.cos() + (y - y0) * angle.sin();
                let across = -(x - x0) * angle.sin() + (y - y0) * angle.cos();
                let envelope = (-along * along / (2.0 * width * width)
                    - across * across / (2.0 * length * length))
                    .exp();
                let average = (-(frequency * width).powi(2) / 2.0).exp();
                envelope * ((frequency * along).cos() - average)
            }
            ReceptiveField::CenterSurround {
                off_center,
                x: x0,
                y: y0,
            } => {
                let r2 = (x - x0).powi(2) + (y - y0).powi(2);
                let (center, surround) = (100.0, 20.0);
                let weight = (-center * r2).exp() - surround / center * (-surround * r2).exp();
                if off_center {
                    -weight
                } else {
                    weight
                }
            }
            ReceptiveField::Edge { angle } => {
                if x * angle.cos() + y * angle.sin() < 0.0 {
                    -1.0
                } else {
                    1.0
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceptiveFieldCell {
    pub field: ReceptiveField,
    pub sensitivity: Hertz,
    #[serde(skip)]
    pub seen_at: Option<Milliseconds>,
    #[serde(skip)]
    pub response: f64,
}

impl Default for ReceptiveFieldCell {
    fn default() -> Self {
        Self {
            field: ReceptiveField::default(),
            sensitivity: Hertz(1000.0),
            seen_at: None,
            response: 0.0,
        }
    }
}

impl ReceptiveFieldCell {
    pub fn weights(&self) -> Vec<f64> {
        let weights: Vec<f64> = (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| self.field.weight(grid(column), grid(row)))
            .collect();
        let mean = weights.iter().sum::<f64>() / weights.len() as f64;
        weights.into_iter().map(|weight| weight - mean).collect()
    }

    pub fn respond(&self, picture: &[f64]) -> f64 {
        picture
            .iter()
            .zip(&self.weights())
            .map(|(brightness, weight)| (2.0 * brightness - 1.0) * weight)
            .sum::<f64>()
            / picture.len() as f64
    }
}

pub fn beside_the_picture(center: Vec3) -> Vec3 {
    center + Vec3::Z * (VISUAL_INPUT_SIZE + RECEPTIVE_FIELD_SIZE + 2.0 * FIELD_GAP) * 0.5
}

pub fn watched_picture(world: &World, cell: Entity) -> Option<Entity> {
    world
        .query::<&Connection>()
        .iter()
        .find(|connection| connection.to == cell && is_picture_wire(world, connection))
        .map(|connection| connection.from)
}

pub fn respond(world: &mut World, engine: &mut Engine, time: Milliseconds) {
    let watched: HashMap<Entity, Entity> = world
        .query::<&Connection>()
        .iter()
        .filter(|connection| is_picture_wire(world, connection))
        .map(|connection| (connection.to, connection.from))
        .collect();
    let stale: Vec<(Entity, Option<Vec<f64>>)> = world
        .query::<(Entity, &ReceptiveFieldCell)>()
        .iter()
        .filter(|(_, field)| {
            field
                .seen_at
                .is_none_or(|seen_at| !(seen_at..seen_at + FRAME_INTERVAL).contains(&time))
        })
        .map(|(cell, _)| {
            let picture = watched.get(&cell).and_then(|&picture| {
                world
                    .get::<&VisualInput>(picture)
                    .ok()
                    .map(|input| input.picture())
            });
            (cell, picture)
        })
        .collect();
    for (cell, picture) in stale {
        if let Ok(mut field) = world.get::<&mut ReceptiveFieldCell>(cell) {
            field.response = picture.map_or(0.0, |picture| field.respond(&picture));
            field.seen_at = Some(time);
        }
    }
    let rates: Vec<(Entity, Hertz)> = world
        .query::<(Entity, &ReceptiveFieldCell)>()
        .iter()
        .map(|(cell, field)| (cell, (field.sensitivity * field.response).max(Hertz::ZERO)))
        .collect();
    for (cell, rate) in rates {
        engine.set_rate(cell, rate);
    }
}

pub fn bound(world: &mut World) {
    for cell in world.query_mut::<&mut ReceptiveFieldCell>() {
        let default = ReceptiveFieldCell::default();
        within(&mut cell.sensitivity, SENSITIVITIES, default.sensitivity);
        match &mut cell.field {
            ReceptiveField::Orientation { angle, x, y } => {
                *angle = turned(*angle);
                within(x, OFFSETS, 0.0);
                within(y, OFFSETS, 0.0);
            }
            ReceptiveField::CenterSurround { x, y, .. } => {
                within(x, OFFSETS, 0.0);
                within(y, OFFSETS, 0.0);
            }
            ReceptiveField::Edge { angle } => *angle = turned(*angle),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;
    use crate::stimuli::visual_input::{spawn_picture_wire, Pattern};
    use crate::stimuli::GeneratorDynamics;
    use neuronify_engine::units::Seconds;

    #[derive(Clone)]
    struct Seen {
        picture: VisualInput,
        cell: ReceptiveFieldCell,
    }

    impl Seen {
        fn picture(&self) -> Vec<f64> {
            self.picture.picture()
        }
    }

    fn input(field: ReceptiveField, pattern: Pattern, pattern_angle: f64) -> Seen {
        Seen {
            picture: VisualInput {
                pattern,
                pattern_angle,
                ..Default::default()
            },
            cell: ReceptiveFieldCell {
                field,
                ..Default::default()
            },
        }
    }

    fn at_phase(seen: &Seen, phase: f64) -> f64 {
        let picture = VisualInput {
            phase,
            ..seen.picture.clone()
        };
        seen.cell.respond(&picture.picture())
    }

    fn mean_response(seen: &Seen) -> f64 {
        (0..200)
            .map(|step| at_phase(seen, step as f64 / 200.0))
            .sum::<f64>()
            / 200.0
    }

    fn spikes(seen: Seen, seconds: f64) -> (usize, f64) {
        let mut world = World::new();
        let picture = world.spawn((seen.picture,));
        let cell = world.spawn((seen.cell, GeneratorDynamics::default()));
        spawn_picture_wire(&mut world, picture, cell);
        let spikes =
            crate::simulation::run_headless(&mut world, Seconds(seconds).to_milliseconds()).len();
        let phase = world.get::<&VisualInput>(picture).unwrap().phase;
        (spikes, phase)
    }

    #[test]
    fn an_orientation_field_prefers_stripes_at_its_own_angle() {
        let field = ReceptiveField::Orientation {
            angle: 0.0,
            x: 0.0,
            y: 0.0,
        };
        let matching = input(field, Pattern::Bar, 0.0);
        let across = input(field, Pattern::Bar, FRAC_PI_2);
        let peak = |input: &Seen| {
            (0..200)
                .map(|step| at_phase(input, step as f64 / 200.0))
                .fold(f64::MIN, f64::max)
        };
        assert!(peak(&matching) > 4.0 * peak(&across).max(1e-3));
    }

    #[test]
    fn an_on_center_field_answers_a_dot_in_its_center_and_off_center_the_opposite() {
        let center = |off_center| ReceptiveField::CenterSurround {
            off_center,
            x: 0.25,
            y: 0.0,
        };
        let on = input(center(false), Pattern::Dot, 0.0);
        let off = input(center(true), Pattern::Dot, 0.0);
        assert!(at_phase(&on, 0.0) > 0.0);
        assert!(at_phase(&off, 0.0) < 0.0);
        assert!(at_phase(&on, 0.0) > at_phase(&on, 0.5));
    }

    #[test]
    fn a_uniform_picture_gives_no_response_whatever_the_field() {
        let fields = [
            ReceptiveField::default(),
            ReceptiveField::Orientation {
                angle: 0.7,
                x: 0.2,
                y: -0.1,
            },
            ReceptiveField::CenterSurround {
                off_center: false,
                x: 0.0,
                y: 0.0,
            },
            ReceptiveField::CenterSurround {
                off_center: true,
                x: 0.25,
                y: 0.0,
            },
            ReceptiveField::Edge { angle: 0.0 },
        ];
        for field in fields {
            let dark = input(field, Pattern::Bar, 0.0);
            assert!(dark.picture().iter().all(|&brightness| brightness == 0.0));
            assert!(at_phase(&dark, 0.0).abs() < 1e-12, "{field:?}");
        }
    }

    #[test]
    fn nothing_outside_a_field_makes_it_fire() {
        let on = input(
            ReceptiveField::CenterSurround {
                off_center: false,
                x: 0.25,
                y: 0.0,
            },
            Pattern::Dot,
            0.0,
        );
        let across = input(ReceptiveField::default(), Pattern::Bar, FRAC_PI_2);
        for step in 0..200 {
            let phase = step as f64 / 200.0;
            if phase > 0.3 && phase < 0.7 {
                assert!(at_phase(&on, phase) <= 0.0, "{phase}");
            }
            assert!(at_phase(&across, phase) < 1e-9, "{phase}");
        }
    }

    #[test]
    fn an_off_center_field_answers_the_dot_crossing_its_surround() {
        let off = input(
            ReceptiveField::CenterSurround {
                off_center: true,
                x: 0.0,
                y: 0.0,
            },
            Pattern::Dot,
            0.0,
        );
        let peak = (0..200)
            .map(|step| at_phase(&off, step as f64 / 200.0))
            .fold(f64::MIN, f64::max);
        assert!(peak > 0.0, "{peak}");
    }

    #[test]
    fn an_edge_field_is_bright_on_one_side_and_dark_on_the_other() {
        let edge = ReceptiveField::Edge { angle: 0.0 };
        assert_eq!(edge.weight(-0.3, 0.0), -1.0);
        assert_eq!(edge.weight(0.3, 0.0), 1.0);
        assert!(mean_response(&input(edge, Pattern::Grating, 0.0)).abs() < 0.05);
    }

    #[test]
    fn an_edge_field_faces_the_same_way_as_a_picture_at_its_angle() {
        for quarter in 0..4 {
            let angle = quarter as f64 * FRAC_PI_2;
            let edge = ReceptiveField::Edge { angle };
            let (x, y) = (0.3 * angle.cos(), 0.3 * angle.sin());
            assert_eq!(edge.weight(x, y), 1.0, "{angle}");
            assert_eq!(edge.weight(-x, -y), -1.0, "{angle}");
        }
    }

    #[test]
    fn a_visual_input_fires_more_the_better_the_picture_matches() {
        let rate = |pattern_angle: f64| {
            spikes(
                input(ReceptiveField::default(), Pattern::Grating, pattern_angle),
                2.0,
            )
            .0
        };
        assert!(
            rate(0.0) > 3 * rate(FRAC_PI_2).max(1),
            "{} {}",
            rate(0.0),
            rate(FRAC_PI_2)
        );
    }

    #[test]
    fn a_field_without_a_picture_never_fires() {
        let mut world = World::new();
        world.spawn((ReceptiveFieldCell::default(), GeneratorDynamics::default()));
        assert!(
            crate::simulation::run_headless(&mut world, Seconds(1.0).to_milliseconds()).is_empty()
        );
    }

    #[test]
    fn two_fields_wired_to_one_picture_each_answer_their_own_spot() {
        let mut world = World::new();
        let picture = world.spawn((VisualInput {
            pattern: Pattern::Dot,
            speed: Hertz::ZERO,
            ..Default::default()
        },));
        let spot = |x: f64| ReceptiveFieldCell {
            field: ReceptiveField::CenterSurround {
                off_center: false,
                x,
                y: 0.0,
            },
            ..Default::default()
        };
        let under_the_dot = world.spawn((spot(0.25), GeneratorDynamics::default()));
        let across = world.spawn((spot(-0.25), GeneratorDynamics::default()));
        for cell in [under_the_dot, across] {
            spawn_picture_wire(&mut world, picture, cell);
        }
        crate::simulation::run_headless(&mut world, Milliseconds(1.0));
        let response = |cell| world.get::<&ReceptiveFieldCell>(cell).unwrap().response;
        assert!(response(under_the_dot) > 0.0);
        assert!(response(across) <= 0.0);
        assert_eq!(watched_picture(&world, across), Some(picture));
    }

    #[test]
    fn a_stopped_bar_keeps_the_input_firing_only_where_it_crosses_the_field() {
        let stopped_at = |phase: f64| {
            let mut seen = input(ReceptiveField::default(), Pattern::Bar, 0.0);
            seen.picture.speed = Hertz::ZERO;
            seen.picture.phase = phase;
            spikes(seen, 1.0).0
        };
        let crossing = stopped_at(0.5);
        let beside = stopped_at(0.1);
        assert!(crossing > 20, "{crossing}");
        assert!(crossing > 10 * beside.max(1), "{crossing} {beside}");
    }
}
