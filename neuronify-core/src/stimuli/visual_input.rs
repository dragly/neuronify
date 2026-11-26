pub mod drawing;
pub mod gui;

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Milliseconds};
use serde::{Deserialize, Serialize};

use crate::components::{Connection, Deletable, Position};
use crate::measurement::frame::{Corner, Frame};
use crate::serialization::bounds::{turned, within};
use crate::stimuli::receptive_field::ReceptiveFieldCell;
use crate::stimuli::About;

pub const RESOLUTION: usize = 20;
pub const VISUAL_INPUT_SIZE: f32 = 8.0;
pub const FRAME_INTERVAL: Milliseconds = Milliseconds(1.0);
pub const DEFAULT_PICTURE_SPEED: Hertz = Hertz(10.0);
pub const DIRECTION_BADGE_RADIUS: f32 = 1.0;
const DIRECTION_BADGE_REACH: f32 = 0.4;
pub const PICTURE_SPEEDS: RangeInclusive<Hertz> = Hertz(-1000.0)..=Hertz(1000.0);
pub const PICTURE_PHASES: RangeInclusive<f64> = 0.0..=1.0;

pub const ABOUT: About = About {
    name: "Visual input",
    description: "Shows a moving picture. Connect it to receptive fields to let them see it.",
    size: Some(VISUAL_INPUT_SIZE),
};

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

pub fn grid(index: usize) -> f64 {
    -0.5 + index as f64 / (RESOLUTION - 1) as f64
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VisualInput {
    pub pattern: Pattern,
    pub pattern_angle: f64,
    pub speed: Hertz,
    pub phase: f64,
}

impl Default for VisualInput {
    fn default() -> Self {
        Self {
            pattern: Pattern::default(),
            pattern_angle: 0.0,
            speed: DEFAULT_PICTURE_SPEED,
            phase: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Along(Vec3),
    Clockwise,
    Counterclockwise,
}

impl VisualInput {
    pub fn reverse(&mut self) {
        self.speed = -self.speed;
    }

    pub fn motion(&self) -> Option<Motion> {
        if self.speed.0 == 0.0 {
            return None;
        }
        let forward = self.speed.0 > 0.0;
        Some(match self.pattern {
            Pattern::Bar | Pattern::Grating => {
                let (sin, cos) = self.pattern_angle.sin_cos();
                let along = Vec3::Z * cos as f32 - Vec3::X * sin as f32;
                Motion::Along(if forward { along } else { -along })
            }
            Pattern::Dot if forward => Motion::Clockwise,
            Pattern::Dot => Motion::Counterclockwise,
        })
    }

    pub fn picture(&self) -> Vec<f64> {
        (0..RESOLUTION)
            .flat_map(|row| (0..RESOLUTION).map(move |column| (column, row)))
            .map(|(column, row)| {
                self.pattern
                    .brightness(self.pattern_angle, grid(column), grid(row), self.phase)
            })
            .collect()
    }
}

pub fn direction_badge(center: Vec3) -> Vec3 {
    Frame::square(center, VISUAL_INPUT_SIZE).corner(Corner::TopRight)
}

pub fn direction_badge_at(world: &World, position: Vec3) -> Option<Entity> {
    world
        .query::<(Entity, &Position, &VisualInput)>()
        .iter()
        .filter(|(_, _, input)| input.motion().is_some())
        .map(|(picture, center, _)| {
            let badge = direction_badge(center.position);
            (picture, badge.with_y(0.0).distance(position.with_y(0.0)))
        })
        .filter(|&(_, distance)| distance < DIRECTION_BADGE_RADIUS + DIRECTION_BADGE_REACH)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(picture, _)| picture)
}

pub fn is_picture_wire(world: &World, connection: &Connection) -> bool {
    world.satisfies::<&VisualInput>(connection.from)
        && world.satisfies::<&ReceptiveFieldCell>(connection.to)
}

pub fn spawn_picture_wire(world: &mut World, picture: Entity, cell: Entity) -> Entity {
    world.spawn((
        Connection {
            from: picture,
            to: cell,
            strength: 1.0,
            directional: false,
        },
        Deletable {},
    ))
}

pub fn advance(world: &mut World, duration: Milliseconds) {
    for input in world.query_mut::<&mut VisualInput>() {
        input.phase = (input.phase + input.speed.cycles_in(duration)).rem_euclid(1.0);
    }
}

pub fn bound(world: &mut World) {
    for input in world.query_mut::<&mut VisualInput>() {
        let default = VisualInput::default();
        within(&mut input.speed, PICTURE_SPEEDS, default.speed);
        within(&mut input.phase, PICTURE_PHASES, default.phase);
        input.pattern_angle = turned(input.pattern_angle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neuronify_engine::units::Seconds;

    fn phase_after(picture: VisualInput, seconds: f64) -> f64 {
        let mut world = World::new();
        let picture = world.spawn((picture,));
        crate::simulation::run_headless(&mut world, Seconds(seconds).to_milliseconds());
        let phase = world.get::<&VisualInput>(picture).unwrap().phase;
        phase
    }

    #[test]
    fn the_picture_moves_as_fast_as_its_speed_and_stands_still_at_zero() {
        let phase_after_a_quarter_second = |speed: f64| {
            let picture = VisualInput {
                speed: Hertz(speed),
                ..Default::default()
            };
            phase_after(picture, 0.25)
        };
        assert!((phase_after_a_quarter_second(0.5) - 0.125).abs() < 1e-6);
        assert!((phase_after_a_quarter_second(2.0) - 0.5).abs() < 1e-6);
        assert_eq!(phase_after_a_quarter_second(0.0), 0.0);
    }

    #[test]
    fn a_reversed_picture_turns_around_where_it_is_and_runs_back() {
        let mut picture = VisualInput {
            speed: Hertz(2.0),
            phase: 0.3,
            ..Default::default()
        };
        let before = picture.picture();
        picture.reverse();
        assert_eq!(picture.picture(), before);
        assert!((phase_after(picture.clone(), 0.1) - 0.1).abs() < 1e-6);
        assert!((phase_after(picture, 0.25) - 0.8).abs() < 1e-6);
    }

    fn lit_center(input: &VisualInput, phase: f64) -> Vec3 {
        let picture = VisualInput {
            phase,
            ..input.clone()
        }
        .picture();
        let lit: Vec<Vec3> = picture
            .iter()
            .enumerate()
            .filter(|(_, &brightness)| brightness > 0.99)
            .map(|(index, _)| {
                drawing::tile_position(Vec3::ZERO, index % RESOLUTION, index / RESOLUTION)
                    .with_y(0.0)
            })
            .collect();
        lit.iter().sum::<Vec3>() / lit.len() as f32
    }

    #[test]
    fn the_arrow_points_the_way_the_drawn_picture_moves() {
        for pattern in [Pattern::Bar, Pattern::Dot] {
            for angle in [0.0, 0.7] {
                for speed in [Hertz(3.0), Hertz(-3.0)] {
                    let input = VisualInput {
                        pattern,
                        pattern_angle: angle,
                        speed,
                        phase: 0.0,
                    };
                    let phase = 0.3;
                    let step = 0.05 * speed.0.signum();
                    let here = lit_center(&input, phase);
                    let moved = lit_center(&input, phase + step) - here;
                    let case = format!("{pattern:?} {angle} {speed:?} {moved}");
                    match input.motion().unwrap() {
                        Motion::Along(direction) => {
                            assert!(moved.normalize().dot(direction) > 0.95, "{case}")
                        }
                        Motion::Clockwise | Motion::Counterclockwise => {
                            let turn = here.cross(moved).y;
                            let clockwise = input.motion() == Some(Motion::Clockwise);
                            assert_eq!(turn < 0.0, clockwise, "{case}");
                        }
                    }
                }
            }
        }
        let still = VisualInput {
            speed: Hertz(0.0),
            ..Default::default()
        };
        assert_eq!(still.motion(), None);
    }

    #[test]
    fn the_badge_on_the_top_right_corner_is_found_and_still_pictures_have_none() {
        let mut world = World::new();
        let center = Vec3::new(2.0, 0.0, -3.0);
        let picture = world.spawn((Position { position: center }, VisualInput::default()));
        let badge = direction_badge(center);
        assert!(badge.x > center.x && badge.z > center.z);
        assert_eq!(direction_badge_at(&world, badge), Some(picture));
        assert_eq!(direction_badge_at(&world, center), None);
        world.get::<&mut VisualInput>(picture).unwrap().speed = Hertz(0.0);
        assert_eq!(direction_badge_at(&world, badge), None);
    }
}
