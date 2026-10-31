use std::f64::consts::{FRAC_PI_2, PI, TAU};

use glam::Vec3;
use hecs::World;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::components::*;
use crate::measurement::voltmeter::Frame;

pub const RESOLUTION: usize = 20;
pub const VISUAL_INPUT_SIZE: f32 = 8.0;
pub const FRAME_INTERVAL: f64 = 0.01;

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

    pub fn brightness(self, angle: f64, x: f64, y: f64, time: f64) -> f64 {
        let across = x * angle.cos() + y * angle.sin();
        match self {
            Pattern::Bar => {
                let center = (time * 0.7).rem_euclid(1.4) - 0.7;
                if (across - center).abs() < 0.075 {
                    1.0
                } else {
                    0.0
                }
            }
            Pattern::Grating => 0.5 + 0.5 * (TAU * (3.0 * across - 0.5 * time)).sin(),
            Pattern::Dot => {
                let (dx, dy) = (x - 0.25 * (PI * time).cos(), y - 0.25 * (PI * time).sin());
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
                let along = (x - x0) * angle.cos() + (y - y0) * angle.sin();
                let across = -(x - x0) * angle.sin() + (y - y0) * angle.cos();
                (-along * along / (2.0 * 0.1 * 0.1) - across * across / (2.0 * 0.2 * 0.2)).exp()
                    * (20.0 * along).cos()
            }
            ReceptiveField::CenterSurround {
                off_center,
                x: x0,
                y: y0,
            } => {
                let r2 = (x - x0).powi(2) + (y - y0).powi(2);
                let weight = (-100.0 * r2).exp() - 0.3 * (-20.0 * r2).exp();
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
            seen_at: None,
            response: 0.0,
        }
    }
}

impl VisualInput {
    pub fn picture(&self, time: f64) -> Vec<f64> {
        (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| {
                self.pattern
                    .brightness(self.pattern_angle, grid(column), grid(row), time)
            })
            .collect()
    }

    pub fn field_weights(&self) -> Vec<f64> {
        (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| self.field.weight(grid(column), grid(row)))
            .collect()
    }

    pub fn respond(&self, time: f64) -> f64 {
        let picture = self.picture(time);
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
        let stale = input
            .seen_at
            .is_none_or(|seen_at| !(seen_at..seen_at + FRAME_INTERVAL).contains(&time));
        if stale {
            input.response = input.respond(time);
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

    fn mean_response(input: &VisualInput) -> f64 {
        (0..200)
            .map(|step| input.respond(step as f64 * 0.01))
            .sum::<f64>()
            / 200.0
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
                .map(|step| input.respond(step as f64 * 0.01))
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
        assert!(on.respond(0.0) > 0.0);
        assert!(off.respond(0.0) < 0.0);
        assert!(on.respond(0.0) > on.respond(1.0));
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
            let mut world = World::new();
            world.spawn((
                input(ReceptiveField::default(), Pattern::Grating, pattern_angle),
                GeneratorDynamics::default(),
            ));
            let mut spikes = 0;
            for step in 0..20_000 {
                see(&mut world, 1e-4, step as f64 * 1e-4);
                for (_, dynamics) in world.query_mut::<&mut GeneratorDynamics>() {
                    spikes += usize::from(dynamics.fired);
                    dynamics.fired = false;
                }
            }
            spikes
        };
        assert!(
            rate(0.0) > 3 * rate(FRAC_PI_2).max(1),
            "{} {}",
            rate(0.0),
            rate(FRAC_PI_2)
        );
    }
}
