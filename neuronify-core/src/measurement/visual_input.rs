use std::f64::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use neuronify_engine::units::{Hertz, Milliseconds};
use serde::{Deserialize, Serialize};

use crate::components::{Connection, Deletable, Position};
use crate::measurement::frame::{Corner, Frame};

pub const RESOLUTION: usize = 20;
pub const VISUAL_INPUT_SIZE: f32 = 8.0;
pub const RECEPTIVE_FIELD_SIZE: f32 = 4.0;
const FIELD_GAP: f32 = 1.5;
pub const FRAME_INTERVAL: Milliseconds = Milliseconds(1.0);
pub const DEFAULT_PICTURE_SPEED: Hertz = Hertz(10.0);
pub const DIRECTION_BADGE_RADIUS: f32 = 1.0;
const DIRECTION_BADGE_REACH: f32 = 0.4;

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

pub fn visual_input_frame(center: Vec3) -> Frame {
    square_frame(center, VISUAL_INPUT_SIZE)
}

pub fn receptive_field_frame(center: Vec3) -> Frame {
    square_frame(center, RECEPTIVE_FIELD_SIZE)
}

fn square_frame(center: Vec3, size: f32) -> Frame {
    Frame {
        bottom_left: center - Vec3::new(size, 0.0, size) * 0.5,
        width: size,
        height: size,
    }
}

pub fn framed_size(world: &World, entity: Entity) -> Option<f32> {
    if world.satisfies::<&VisualInput>(entity) {
        Some(VISUAL_INPUT_SIZE)
    } else if world.satisfies::<&ReceptiveFieldCell>(entity) {
        Some(RECEPTIVE_FIELD_SIZE)
    } else {
        None
    }
}

pub fn edge_toward(center: Vec3, toward: Vec3, size: f32) -> Option<Vec3> {
    let offset = toward - center;
    let reach = offset.x.abs().max(offset.z.abs());
    (reach > size * 0.5).then(|| center + offset * (size * 0.5 / reach))
}

pub fn beside_the_picture(center: Vec3) -> Vec3 {
    center + Vec3::Z * (VISUAL_INPUT_SIZE + RECEPTIVE_FIELD_SIZE + 2.0 * FIELD_GAP) * 0.5
}

pub fn direction_badge(center: Vec3) -> Vec3 {
    visual_input_frame(center).corner(Corner::TopRight)
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

pub fn watched_picture(world: &World, cell: Entity) -> Option<Entity> {
    world
        .query::<&Connection>()
        .iter()
        .find(|connection| connection.to == cell && is_picture_wire(world, connection))
        .map(|connection| connection.from)
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

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;
    use crate::components::GeneratorDynamics;
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
        let cell = world.spawn((seen.cell, crate::components::GeneratorDynamics::default()));
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
    fn the_picture_moves_as_fast_as_its_speed_and_stands_still_at_zero() {
        let phase_after_a_quarter_second = |speed: f64| {
            let mut seen = input(ReceptiveField::default(), Pattern::Bar, 0.0);
            seen.picture.speed = Hertz(speed);
            spikes(seen, 0.25).1
        };
        assert!((phase_after_a_quarter_second(0.5) - 0.125).abs() < 1e-6);
        assert!((phase_after_a_quarter_second(2.0) - 0.5).abs() < 1e-6);
        assert_eq!(phase_after_a_quarter_second(0.0), 0.0);
    }

    #[test]
    fn a_reversed_picture_turns_around_where_it_is_and_runs_back() {
        let mut seen = input(ReceptiveField::default(), Pattern::Bar, 0.0);
        seen.picture.speed = Hertz(2.0);
        seen.picture.phase = 0.3;
        let before = seen.picture();
        seen.picture.reverse();
        assert_eq!(seen.picture(), before);
        assert!((spikes(seen.clone(), 0.1).1 - 0.1).abs() < 1e-6);
        assert!((spikes(seen, 0.25).1 - 0.8).abs() < 1e-6);
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
                crate::rendering::visual_input::tile_position(
                    Vec3::ZERO,
                    index % RESOLUTION,
                    index / RESOLUTION,
                )
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
