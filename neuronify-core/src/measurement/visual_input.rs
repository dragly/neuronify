use std::f64::consts::{FRAC_PI_2, TAU};

use glam::Vec3;
use hecs::World;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::voltmeter::Frame;

pub const RESOLUTION: usize = 20;
pub const VISUAL_INPUT_SIZE: f32 = 8.0;
pub const FRAME_INTERVAL: f64 = 0.001;
pub const DEFAULT_PICTURE_SPEED: f64 = 10.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Pattern {
    #[default]
    Bar,
    Grating,
    Dot,
}

impl Pattern {
    pub const ALL: [Pattern; 3] = [Pattern::Bar, Pattern::Grating, Pattern::Dot];

    pub fn name(self) -> &'static str {
        match self {
            Pattern::Bar => "Bar",
            Pattern::Grating => "Stripes",
            Pattern::Dot => "Dot",
        }
    }

    pub fn brightness(self, angle: f64, x: f64, y: f64, phase: f64) -> f64 {
        let across = x * angle.cos() + y * angle.sin();
        match self {
            Pattern::Bar => {
                let center = 1.4 * phase - 0.7;
                if (across - center).abs() < 0.075 {
                    1.0
                } else {
                    0.0
                }
            }
            Pattern::Grating => 0.5 + 0.5 * (TAU * (3.0 * across - phase)).sin(),
            Pattern::Dot => {
                let (dx, dy) = (
                    x - 0.25 * (TAU * phase).cos(),
                    y - 0.25 * (TAU * phase).sin(),
                );
                if dx * dx + dy * dy < 0.12 * 0.12 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

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
        ReceptiveField::Edge { angle: FRAC_PI_2 },
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
                if x * angle.sin() + y * angle.cos() < 0.0 {
                    -1.0
                } else {
                    1.0
                }
            }
        }
    }
}

pub fn grid(index: usize) -> f64 {
    -0.5 + index as f64 / (RESOLUTION - 1) as f64
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VisualInput {
    pub field: ReceptiveField,
    pub sensitivity: f64,
    pub show_field: bool,
    pub pattern: Pattern,
    pub pattern_angle: f64,
    pub speed: f64,
    pub phase: f64,
    #[serde(skip)]
    pub seen_at: Option<f64>,
    #[serde(skip)]
    pub response: f64,
}

impl Default for VisualInput {
    fn default() -> Self {
        Self {
            field: ReceptiveField::default(),
            sensitivity: 1000.0,
            show_field: true,
            pattern: Pattern::default(),
            pattern_angle: 0.0,
            speed: DEFAULT_PICTURE_SPEED,
            phase: 0.0,
            seen_at: None,
            response: 0.0,
        }
    }
}

impl VisualInput {
    pub fn picture(&self) -> Vec<f64> {
        (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| {
                self.pattern
                    .brightness(self.pattern_angle, grid(column), grid(row), self.phase)
            })
            .collect()
    }

    pub fn field_weights(&self) -> Vec<f64> {
        let weights: Vec<f64> = (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| self.field.weight(grid(column), grid(row)))
            .collect();
        let mean = weights.iter().sum::<f64>() / weights.len() as f64;
        weights.into_iter().map(|weight| weight - mean).collect()
    }

    pub fn respond(&self) -> f64 {
        let picture = self.picture();
        let weights = self.field_weights();
        picture
            .iter()
            .zip(&weights)
            .map(|(brightness, weight)| (2.0 * brightness - 1.0) * weight)
            .sum::<f64>()
            / picture.len() as f64
    }
}

pub fn visual_input_frame(center: Vec3) -> Frame {
    Frame {
        bottom_left: center - Vec3::new(VISUAL_INPUT_SIZE, 0.0, VISUAL_INPUT_SIZE) * 0.5,
        width: VISUAL_INPUT_SIZE,
        height: VISUAL_INPUT_SIZE,
    }
}

pub fn see(world: &mut World, dt: f64, time: f64) {
    let mut rng = rand::thread_rng();
    for (_, (input, dynamics)) in world.query_mut::<(&mut VisualInput, &mut GeneratorDynamics)>() {
        dynamics.time_since_fire += dt;
        input.phase = (input.phase + input.speed * dt).rem_euclid(1.0);
        let stale = input
            .seen_at
            .is_none_or(|seen_at| !(seen_at..seen_at + FRAME_INTERVAL).contains(&time));
        if stale {
            input.response = input.respond();
            input.seen_at = Some(time);
        }
        if rng.gen::<f64>() < input.sensitivity * input.response * dt {
            dynamics.fired = true;
            dynamics.time_since_fire = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(field: ReceptiveField, pattern: Pattern, pattern_angle: f64) -> VisualInput {
        VisualInput {
            field,
            pattern,
            pattern_angle,
            ..Default::default()
        }
    }

    fn at_phase(input: &VisualInput, phase: f64) -> f64 {
        VisualInput {
            phase,
            ..input.clone()
        }
        .respond()
    }

    fn mean_response(input: &VisualInput) -> f64 {
        (0..200)
            .map(|step| at_phase(input, step as f64 / 200.0))
            .sum::<f64>()
            / 200.0
    }

    fn spikes(input: VisualInput, seconds: f64) -> (usize, f64) {
        let mut world = World::new();
        let entity = world.spawn((input, GeneratorDynamics::default()));
        let mut spikes = 0;
        let steps = (seconds / 1e-4) as usize;
        for step in 0..steps {
            see(&mut world, 1e-4, step as f64 * 1e-4);
            for (_, dynamics) in world.query_mut::<&mut GeneratorDynamics>() {
                spikes += usize::from(dynamics.fired);
                dynamics.fired = false;
            }
        }
        let phase = world.get::<&VisualInput>(entity).unwrap().phase;
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
        let peak = |input: &VisualInput| {
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
            ReceptiveField::Edge { angle: FRAC_PI_2 },
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
        let edge = ReceptiveField::Edge { angle: FRAC_PI_2 };
        assert_eq!(edge.weight(-0.3, 0.0), -1.0);
        assert_eq!(edge.weight(0.3, 0.0), 1.0);
        assert!(mean_response(&input(edge, Pattern::Grating, 0.0)).abs() < 0.05);
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
    fn the_picture_moves_as_fast_as_its_speed_and_stands_still_at_zero() {
        let phase_after_a_quarter_second = |speed: f64| {
            spikes(
                VisualInput {
                    speed,
                    ..Default::default()
                },
                0.25,
            )
            .1
        };
        assert!((phase_after_a_quarter_second(0.5) - 0.125).abs() < 1e-6);
        assert!((phase_after_a_quarter_second(2.0) - 0.5).abs() < 1e-6);
        assert_eq!(phase_after_a_quarter_second(0.0), 0.0);
    }

    #[test]
    fn a_stopped_bar_keeps_the_input_firing_only_where_it_crosses_the_field() {
        let stopped_at = |phase: f64| {
            spikes(
                VisualInput {
                    speed: 0.0,
                    phase,
                    ..input(ReceptiveField::default(), Pattern::Bar, 0.0)
                },
                1.0,
            )
            .0
        };
        let crossing = stopped_at(0.5);
        let beside = stopped_at(0.1);
        assert!(crossing > 20, "{crossing}");
        assert!(crossing > 10 * beside.max(1), "{crossing} {beside}");
    }
}
