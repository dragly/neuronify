use std::collections::BTreeMap;

use glam::Vec3;
use hecs::{Entity, World};

use super::{
    arc_points, bends_only, inside_length, near_neurons, obstacles, octilinear, over_neurons,
    pair_cost, rounded, Shape, CLEARANCE, CROSSING_COST, LANE_GAP, METER_COST, MIN_SEGMENT_LENGTH,
    NEURON_COST, SETTLE_PASSES,
};
use crate::components::*;
use crate::measurement::visual_input::{edge_toward, framed_size, is_picture_wire};
use crate::measurement::voltmeter::Frame;

const SHAPES: [Shape; 3] = [
    Shape::StraightFirst,
    Shape::DiagonalFirst,
    Shape::DiagonalBetween,
];
const PORT_MARGIN: f32 = LANE_GAP;
const SETTLED: f32 = 1e-3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    fn normal(self) -> Vec3 {
        match self {
            Side::Top => Vec3::X,
            Side::Bottom => -Vec3::X,
            Side::Left => -Vec3::Z,
            Side::Right => Vec3::Z,
        }
    }

    fn along(self) -> Vec3 {
        match self {
            Side::Top | Side::Bottom => Vec3::Z,
            Side::Left | Side::Right => Vec3::X,
        }
    }

    fn facing(from: &Square, to: &Square, axis: Vec3) -> Side {
        let ahead = to.center.dot(axis) > from.center.dot(axis);
        match (axis == Vec3::X, ahead) {
            (true, true) => Side::Top,
            (true, false) => Side::Bottom,
            (false, true) => Side::Right,
            (false, false) => Side::Left,
        }
    }
}

#[derive(Clone, Copy)]
struct Square {
    entity: Entity,
    center: Vec3,
    half: f32,
}

impl Square {
    fn of(world: &World, entity: Entity) -> Option<Square> {
        Some(Square {
            entity,
            center: world.get::<&Position>(entity).ok()?.position,
            half: framed_size(world, entity)? * 0.5,
        })
    }

    fn frame(&self) -> Frame {
        Frame {
            bottom_left: self.center - Vec3::new(self.half, 0.0, self.half),
            width: 2.0 * self.half,
            height: 2.0 * self.half,
        }
    }

    fn port(&self, side: Side, along: f32) -> Vec3 {
        self.center + side.normal() * self.half + side.along() * along
    }

    fn reach(&self, axis: Vec3) -> (f32, f32) {
        let middle = self.center.dot(axis);
        (middle - self.half, middle + self.half)
    }

    fn gap(&self, other: &Square, axis: Vec3) -> f32 {
        let (low, high) = self.reach(axis);
        let (other_low, other_high) = other.reach(axis);
        (other_low - high).max(low - other_high)
    }
}

struct Link {
    connection: Entity,
    picture: Square,
    field: Square,
}

#[derive(Clone, Copy)]
struct Plan {
    link: usize,
    sides: (Side, Side),
    along: (f32, f32),
}

pub fn picture_wire_points(world: &World) -> BTreeMap<Entity, Vec<Vec3>> {
    picture_wire_corners(world)
        .into_iter()
        .map(|(connection, corners)| {
            let mut points: Vec<Vec3> = rounded(&corners, Vec3::ZERO)
                .iter()
                .flat_map(arc_points)
                .collect();
            points.dedup_by(|a, b| a.distance(*b) < MIN_SEGMENT_LENGTH);
            (connection, if points.len() < 2 { corners } else { points })
        })
        .collect()
}

pub fn picture_wire_corners(world: &World) -> BTreeMap<Entity, Vec<Vec3>> {
    let links: Vec<Link> = world
        .query::<(Entity, &Connection)>()
        .iter()
        .filter(|(_, connection)| is_picture_wire(world, connection))
        .filter_map(|(entity, connection)| {
            Some(Link {
                connection: entity,
                picture: Square::of(world, connection.from)?,
                field: Square::of(world, connection.to)?,
            })
        })
        .collect();
    let mut corners = BTreeMap::new();
    if links.is_empty() {
        return corners;
    }
    let neurons = obstacles(world);
    let frames = frames(world);
    let mut plans = Vec::new();
    for (index, link) in links.iter().enumerate() {
        let best = side_options(link)
            .into_iter()
            .map(|sides| {
                let plan = Plan {
                    link: index,
                    sides,
                    along: wanted(link, sides),
                };
                let path = route(link, &plan, LANE_GAP, Shape::StraightFirst, &neurons);
                (plan, own_cost(&path, link, &frames, &neurons))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        match best {
            Some((plan, _)) => plans.push(plan),
            None => {
                corners.insert(link.connection, straight_between(link));
            }
        }
    }
    spread(&mut plans, &links);
    let options: Vec<Vec<Vec<Vec3>>> = plans
        .iter()
        .zip(stubs(&plans, &links))
        .map(|(plan, stub)| {
            let mut routes: Vec<Vec<Vec3>> = Vec::new();
            for shape in SHAPES {
                let path = route(&links[plan.link], plan, stub, shape, &neurons);
                if !routes.contains(&path) {
                    routes.push(path);
                }
            }
            routes
        })
        .collect();
    let own: Vec<Vec<f32>> = plans
        .iter()
        .zip(&options)
        .map(|(plan, routes)| {
            routes
                .iter()
                .map(|path| own_cost(path, &links[plan.link], &frames, &neurons))
                .collect()
        })
        .collect();
    for ((plan, routes), choice) in plans.iter().zip(&options).zip(settle(&options, &own)) {
        corners.insert(links[plan.link].connection, routes[choice].clone());
    }
    corners
}

fn frames(world: &World) -> Vec<(Entity, Frame)> {
    let mut frames: Vec<(Entity, Frame)> = world
        .query::<Entity>()
        .with::<&Position>()
        .iter()
        .filter_map(|entity| Some((entity, Square::of(world, entity)?.frame())))
        .collect();
    frames.extend(
        world
            .query::<Entity>()
            .with::<&MeterSize>()
            .iter()
            .filter_map(|meter| Some((meter, Frame::of(world, meter)?))),
    );
    frames
}

fn side_options(link: &Link) -> Vec<(Side, Side)> {
    let (picture, field) = (&link.picture, &link.field);
    let facing = |out: Vec3, into: Vec3| {
        (
            Side::facing(picture, field, out),
            Side::facing(field, picture, into),
        )
    };
    match (
        picture.gap(field, Vec3::X) > 0.0,
        picture.gap(field, Vec3::Z) > 0.0,
    ) {
        (true, false) => vec![facing(Vec3::X, Vec3::X)],
        (false, true) => vec![facing(Vec3::Z, Vec3::Z)],
        (true, true) => vec![facing(Vec3::Z, Vec3::X), facing(Vec3::X, Vec3::Z)],
        (false, false) => Vec::new(),
    }
}

fn wanted(link: &Link, (from, to): (Side, Side)) -> (f32, f32) {
    let (picture, field) = (&link.picture, &link.field);
    let axis = from.along();
    let middle = |square: &Square| square.center.dot(axis);
    let (low, high) = picture.reach(axis);
    if to.along() != axis {
        let at = middle(field).clamp(low + PORT_MARGIN, high - PORT_MARGIN);
        return (at - middle(picture), 0.0);
    }
    let (field_low, field_high) = field.reach(axis);
    let (shared_low, shared_high) = (low.max(field_low), high.min(field_high));
    let at = if shared_high - shared_low > 2.0 * PORT_MARGIN {
        middle(field).clamp(shared_low + PORT_MARGIN, shared_high - PORT_MARGIN)
    } else {
        (shared_low + shared_high) * 0.5
    };
    (at - middle(picture), at - middle(field))
}

fn spread(plans: &mut [Plan], links: &[Link]) {
    let mut groups: BTreeMap<(Entity, Side), Vec<(usize, bool)>> = BTreeMap::new();
    for (index, plan) in plans.iter().enumerate() {
        let link = &links[plan.link];
        groups
            .entry((link.picture.entity, plan.sides.0))
            .or_default()
            .push((index, true));
        groups
            .entry((link.field.entity, plan.sides.1))
            .or_default()
            .push((index, false));
    }
    for ((_, side), ends) in groups {
        if ends.len() < 2 {
            continue;
        }
        let mut keyed: Vec<(f32, Entity, usize, bool, f32, f32)> = ends
            .into_iter()
            .map(|(index, on_picture)| {
                let plan = &plans[index];
                let link = &links[plan.link];
                let (square, along, other) = if on_picture {
                    let other = link.field.port(plan.sides.1, plan.along.1);
                    (link.picture, plan.along.0, other)
                } else {
                    let other = link.picture.port(plan.sides.0, plan.along.0);
                    (link.field, plan.along.1, other)
                };
                let toward = other - square.port(side, 0.0);
                let key = toward.dot(side.along()).atan2(toward.dot(side.normal()));
                (key, link.connection, index, on_picture, along, square.half)
            })
            .collect();
        keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let room = (keyed[0].5 - PORT_MARGIN).max(0.0);
        let wanted: Vec<f32> = keyed.iter().map(|end| end.4).collect();
        let low = wanted.iter().copied().fold(-room, f32::min);
        let high = wanted.iter().copied().fold(room, f32::max);
        for (end, along) in keyed.iter().zip(spaced(&wanted, low, high)) {
            let plan = &mut plans[end.2];
            if end.3 {
                plan.along.0 = along;
            } else {
                plan.along.1 = along;
            }
        }
    }
}

fn spaced(wanted: &[f32], low: f32, high: f32) -> Vec<f32> {
    let count = wanted.len();
    let gap = match count {
        0 | 1 => 0.0,
        _ => LANE_GAP.min((high - low) / (count - 1) as f32),
    };
    let mut placed = wanted.to_vec();
    for index in 0..count {
        placed[index] = placed[index].max(low + gap * index as f32);
        if index > 0 {
            placed[index] = placed[index].max(placed[index - 1] + gap);
        }
    }
    for index in (0..count).rev() {
        placed[index] = placed[index].min(high - gap * (count - 1 - index) as f32);
        if index + 1 < count {
            placed[index] = placed[index].min(placed[index + 1] - gap);
        }
    }
    placed
}

fn stubs(plans: &[Plan], links: &[Link]) -> Vec<f32> {
    let mut groups: BTreeMap<(Entity, Side, bool), Vec<(f32, usize)>> = BTreeMap::new();
    for (index, plan) in plans.iter().enumerate() {
        let link = &links[plan.link];
        let (from, to) = plan.sides;
        let offset = (link.field.port(to, plan.along.1) - link.picture.port(from, plan.along.0))
            .dot(from.along());
        if offset.abs() > SETTLED {
            groups
                .entry((link.picture.entity, from, offset > 0.0))
                .or_default()
                .push((offset.abs(), index));
        }
    }
    let mut levels = vec![0; plans.len()];
    for mut group in groups.into_values() {
        group.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        for (level, (_, index)) in group.into_iter().enumerate() {
            levels[index] = level;
        }
    }
    levels
        .into_iter()
        .map(|level| LANE_GAP * (level + 1) as f32)
        .collect()
}

fn route(link: &Link, plan: &Plan, stub: f32, shape: Shape, neurons: &[Vec3]) -> Vec<Vec3> {
    let (from, to) = plan.sides;
    let start = link.picture.port(from, plan.along.0);
    let end = link.field.port(to, plan.along.1);
    let ahead = (end - start).dot(from.normal()).max(0.0);
    let back = (start - end).dot(to.normal()).max(0.0);
    let (out, into) = if from.along() == to.along() {
        let scale = (ahead / (stub + LANE_GAP)).min(1.0);
        (stub * scale, LANE_GAP * scale)
    } else {
        (stub.min(0.5 * ahead), LANE_GAP.min(0.5 * back))
    };
    let leave = start + from.normal() * out;
    let arrive = end + to.normal() * into;
    let mut points = vec![start, leave];
    points.extend(octilinear(leave, arrive, shape, neurons));
    points.push(end);
    bends_only(points)
}

fn straight_between(link: &Link) -> Vec<Vec3> {
    let (from, to) = (link.picture.center, link.field.center);
    vec![
        edge_toward(from, to, 2.0 * link.picture.half).unwrap_or(from),
        edge_toward(to, from, 2.0 * link.field.half).unwrap_or(to),
    ]
}

fn own_cost(path: &[Vec3], link: &Link, frames: &[(Entity, Frame)], neurons: &[Vec3]) -> f32 {
    let ends = [link.picture.entity, link.field.entity];
    let length: f32 = path.windows(2).map(|pair| pair[0].distance(pair[1])).sum();
    let over_frames: f32 = path
        .windows(2)
        .map(|pair| {
            frames
                .iter()
                .map(|(entity, frame)| {
                    let margin = if ends.contains(entity) {
                        -SETTLED
                    } else {
                        CLEARANCE
                    };
                    inside_length(pair[0], pair[1], frame, margin)
                })
                .sum::<f32>()
        })
        .sum();
    let near = near_neurons(path, path[0], neurons);
    METER_COST * over_frames + NEURON_COST * over_neurons(path, &near) + length
}

fn settle(options: &[Vec<Vec<Vec3>>], own: &[Vec<f32>]) -> Vec<usize> {
    let mut chosen: Vec<usize> = own
        .iter()
        .map(|costs| {
            (0..costs.len())
                .min_by(|&a, &b| costs[a].total_cmp(&costs[b]))
                .unwrap_or(0)
        })
        .collect();
    for _ in 0..SETTLE_PASSES {
        let mut changed = false;
        for wire in 0..options.len() {
            let total = |option: usize, chosen: &[usize]| {
                own[wire][option]
                    + (0..options.len())
                        .filter(|&other| other != wire)
                        .map(|other| {
                            pair_cost(
                                &options[wire][option],
                                &options[other][chosen[other]],
                                CROSSING_COST,
                            )
                        })
                        .sum::<f32>()
            };
            let current = total(chosen[wire], &chosen);
            let best = (0..options[wire].len())
                .map(|option| (option, total(option, &chosen)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((option, _)) = best.filter(|&(_, cost)| cost < current - SETTLED) {
                chosen[wire] = option;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    chosen
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use glam::Vec2;

    use super::*;
    use crate::elements::Element;
    use crate::measurement::visual_input::spawn_picture_wire;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::scene::shown_paths;
    use crate::rendering::{collect_meter_wires, Theme};

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    fn wired(world: &mut World, picture: Entity, fields: &[Vec3]) -> Vec<(Entity, Entity)> {
        fields
            .iter()
            .map(|&position| {
                let field = Element::ReceptiveField.spawn(world, position);
                (spawn_picture_wire(world, picture, field), field)
            })
            .collect()
    }

    fn on_edge(square: &Square, point: Vec3) -> bool {
        let offset = point - square.center;
        let (x, z) = (offset.x.abs(), offset.z.abs());
        let near = |value: f32| (value - square.half).abs() < 1e-4;
        (near(x) && z <= square.half + 1e-4) || (near(z) && x <= square.half + 1e-4)
    }

    fn flat(point: Vec3) -> Vec2 {
        Vec2::new(point.x, point.z)
    }

    fn crosses(first: &[Vec3], second: &[Vec3]) -> bool {
        first.windows(2).any(|a| {
            second.windows(2).any(|b| {
                let (start, along) = (flat(a[0]), flat(a[1]) - flat(a[0]));
                let (other, across) = (flat(b[0]), flat(b[1]) - flat(b[0]));
                let turn = along.perp_dot(across);
                turn.abs() > 1e-6
                    && (0.0..=1.0).contains(&((other - start).perp_dot(across) / turn))
                    && (0.0..=1.0).contains(&((other - start).perp_dot(along) / turn))
            })
        })
    }

    #[test]
    fn a_field_beside_the_picture_is_wired_straight_across_between_the_facing_sides() {
        let mut world = World::new();
        let picture = Element::VisualInput.spawn(&mut world, at(0.0, 0.0));
        let wires = wired(&mut world, picture, &[at(2.8, 10.0), at(-2.8, 10.0)]);
        let corners = picture_wire_corners(&world);
        for ((wire, _), height) in wires.iter().zip([2.8, -2.8]) {
            assert_eq!(corners[wire], vec![at(height, 4.0), at(height, 8.0)]);
        }
    }

    #[test]
    fn fields_in_a_row_below_the_picture_are_wired_without_crossings_or_passing_over_a_field() {
        let mut world = World::new();
        let picture = Element::VisualInput.spawn(&mut world, at(0.0, 0.0));
        let row: Vec<Vec3> = (-2..=2).map(|k| at(-7.2, 5.0 * k as f32)).collect();
        let wires = wired(&mut world, picture, &row);
        let corners = picture_wire_corners(&world);
        let frames = frames(&world);
        let paths: Vec<&Vec<Vec3>> = wires.iter().map(|(wire, _)| &corners[wire]).collect();
        for (path, &(_, field)) in paths.iter().zip(&wires) {
            assert!(on_edge(&Square::of(&world, picture).unwrap(), path[0]));
            assert!(on_edge(
                &Square::of(&world, field).unwrap(),
                path[path.len() - 1]
            ));
            for pair in path.windows(2) {
                let step = (pair[1] - pair[0]).abs();
                assert!(
                    step.x < 1e-4 || step.z < 1e-4 || (step.x - step.z).abs() < 1e-4,
                    "{path:?}"
                );
                for (_, frame) in &frames {
                    assert!(
                        inside_length(pair[0], pair[1], frame, -SETTLED) < 1e-4,
                        "{path:?}"
                    );
                }
            }
        }
        for (index, first) in paths.iter().enumerate() {
            for second in &paths[index + 1..] {
                assert!(!crosses(first, second), "{first:?} {second:?}");
                assert_eq!(pair_cost(first, second, CROSSING_COST), 0.0);
            }
        }
        assert!(paths[1..4].iter().all(|path| path.len() == 2));
        assert_eq!((paths[0][0].z, paths[4][0].z), (-4.0, 4.0));
    }

    #[test]
    fn ports_on_one_side_keep_their_order_and_spacing() {
        assert_eq!(spaced(&[-2.0, 0.0, 2.0], -3.0, 3.0), vec![-2.0, 0.0, 2.0]);
        let placed = spaced(&[1.0, 1.0, 1.0, 0.5], -1.0, 1.0);
        assert!(placed
            .windows(2)
            .all(|pair| pair[1] - pair[0] >= LANE_GAP - 1e-5));
        assert!(placed.iter().all(|along| (-1.0..=1.0).contains(along)));
    }

    #[test]
    fn a_picture_wire_is_drawn_as_a_wire_and_picked_along_its_route() {
        let mut world = World::new();
        let picture = Element::VisualInput.spawn(&mut world, at(0.0, 0.0));
        let [(wire, _)] = wired(&mut world, picture, &[at(-20.0, 12.0)])[..] else {
            unreachable!()
        };
        let palette = Theme::Dark.palette();
        let corners = picture_wire_corners(&world)[&wire].clone();
        assert!(corners.len() > 2);
        let path = &connection_paths(&world, &HashMap::new(), &palette)[&wire];
        assert_eq!(path.points.first(), corners.first());
        assert_eq!(path.points.last(), corners.last());
        assert!(shown_paths(&world, &HashMap::new(), &palette).is_empty());
        let drawn = collect_meter_wires(&world, &palette);
        assert_eq!(drawn.first().map(|arc| arc.start), corners.first().copied());
        assert_eq!(drawn.last().map(|arc| arc.end), corners.last().copied());
    }
}
