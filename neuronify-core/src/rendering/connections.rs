use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use glam::{Quat, Vec2, Vec3};

use crate::components::*;
use crate::constants::NODE_RADIUS;
use crate::measurement::firing_rate::RatePlot;
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::visual_input::VisualInput;
use crate::measurement::voltmeter::{measured, Frame};
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::{ArcData, ConnectionData};

const MIN_SEGMENT_LENGTH: f32 = 1e-4;
const LANE_GAP: f32 = 0.6;
const CLEARANCE: f32 = 0.3;
const NEURON_CLEARANCE: f32 = 1.3;
const OVERLAP_DISTANCE: f32 = 0.5 * LANE_GAP;
const BEND_RADIUS: f32 = 0.8;
const STRAIGHT_TURN: f32 = 1e-4;
const ARC_POINTS: usize = 8;
const SETTLE_PASSES: usize = 4;
const DIAGONAL_SHARES: [f32; 5] = [0.5, 0.35, 0.65, 0.2, 0.8];
const METER_COST: f32 = 100.0;
const NEURON_COST: f32 = 10.0;
const CROSSING_COST: f32 = 60.0;
const FOREIGN_CROSSING_COST: f32 = 10.0;
const OVERLAP_COST: f32 = 30.0;
const FAR_SIDE_COST: f32 = 5.0;
const CACHED_LAYOUTS: usize = 8;
const LAYOUT_TOLERANCE: f32 = 1e-3;

thread_local! {
    static LAYOUTS: RefCell<Vec<Layout>> = const { RefCell::new(Vec::new()) };
}

pub fn collect_meter_wires(world: &hecs::World, palette: &Palette) -> Vec<ArcData> {
    let hidden = hidden_connections(world);
    let meters: Vec<Meter> = world
        .query::<()>()
        .with::<&MeterSize>()
        .iter()
        .filter_map(|(meter, _)| Meter::of(world, meter, &hidden))
        .collect();
    let mut wires: Vec<ArcData> = meter_wire_arcs(&meters, &obstacles(world), palette.neutral)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    for (speaker, position) in world.query::<&Position>().with::<&Loudspeaker>().iter() {
        for (connection, neuron) in measured(world, speaker) {
            if hidden.contains(&connection) {
                continue;
            }
            if let Ok(start) = world.get::<&Position>(neuron) {
                wires.push(straight(start.position, position.position, palette.neutral));
            }
        }
    }
    wires
}

fn obstacles(world: &hecs::World) -> Vec<Vec3> {
    let mut obstacles: Vec<Vec3> = world
        .query::<&Position>()
        .with::<&LeakyNeuron>()
        .iter()
        .map(|(_, position)| position.position)
        .collect();
    obstacles.extend(
        world
            .query::<&Position>()
            .with::<&GeneratorDynamics>()
            .without::<&VisualInput>()
            .iter()
            .map(|(_, position)| position.position),
    );
    obstacles
}

struct Meter {
    frame: Frame,
    rows: usize,
    starts: Vec<(usize, Vec3)>,
}

impl Meter {
    fn of(
        world: &hecs::World,
        meter: hecs::Entity,
        hidden: &HashSet<hecs::Entity>,
    ) -> Option<Meter> {
        let frame = Frame::of(world, meter)?;
        let measured = measured(world, meter);
        let rows = if world.satisfies::<&RatePlot>(meter).unwrap_or(false) {
            1
        } else {
            measured.len()
        };
        let starts = measured
            .iter()
            .enumerate()
            .filter(|(_, (connection, _))| !hidden.contains(connection))
            .filter_map(|(index, (_, neuron))| {
                let position = world.get::<&Position>(*neuron).ok()?.position;
                Some((index.min(rows - 1), position))
            })
            .collect();
        Some(Meter {
            frame,
            rows,
            starts,
        })
    }

    fn row_height(&self, row: usize) -> f32 {
        self.frame.row(row, self.rows).point(0.0, 0.5).x
    }

    fn aim(&self, wire: usize, route: Route) -> f32 {
        let bounds = Bounds::of(&self.frame);
        match route.around {
            Some(Around::Top) => bounds.top,
            Some(Around::Bottom) => bounds.bottom,
            None => self.row_height(self.starts[wire].0),
        }
    }

    fn heading_down(&self, wire: usize, route: Route) -> bool {
        self.aim(wire, route) < self.starts[wire].1.x
    }

    fn routes(&self, wire: usize, neurons: &[Vec3]) -> Vec<Route> {
        let bounds = Bounds::of(&self.frame);
        let start = self.starts[wire].1;
        let facing = if start.z < (bounds.left + bounds.right) / 2.0 {
            Side::Left
        } else {
            Side::Right
        };
        let nearer = if start.x > (bounds.bottom + bounds.top) / 2.0 {
            Around::Top
        } else {
            Around::Bottom
        };
        let above_or_below = (bounds.left..=bounds.right).contains(&start.z)
            && !(bounds.bottom..=bounds.top).contains(&start.x);
        let mut routes = Vec::new();
        if !above_or_below {
            for shape in [
                Shape::StraightFirst,
                Shape::DiagonalFirst,
                Shape::DiagonalBetween,
            ] {
                routes.push(Route {
                    side: facing,
                    around: None,
                    shape,
                    bypass: None,
                });
            }
        }
        let corners: &[Around] = if above_or_below {
            &[nearer]
        } else {
            &[nearer, nearer.opposite()]
        };
        for side in [facing, facing.opposite()] {
            for &around in corners {
                for shape in [Shape::DiagonalFirst, Shape::StraightFirst] {
                    routes.push(Route {
                        side,
                        around: Some(around),
                        shape,
                        bypass: None,
                    });
                }
            }
        }
        let bypasses: Vec<Route> = routes
            .iter()
            .filter(|&&route| {
                blockers(start, self.aim(wire, route), neurons)
                    .next()
                    .is_some()
            })
            .flat_map(|&route| {
                [Side::Left, Side::Right].map(|side| Route {
                    bypass: Some(side),
                    ..route
                })
            })
            .collect();
        routes.extend(bypasses);
        routes
    }

    fn lanes(&self, routes: &[Route]) -> Vec<usize> {
        let inner_first = |wire: usize| {
            let (row, start) = self.starts[wire];
            let from_below = match routes[wire].around {
                Some(Around::Top) => false,
                Some(Around::Bottom) => true,
                None => start.x < self.row_height(row),
            };
            if from_below {
                self.rows - row
            } else {
                row
            }
        };
        let span = |wire: usize| {
            let (row, start) = self.starts[wire];
            let reach = match routes[wire].around {
                Some(_) => self.aim(wire, routes[wire]),
                None => start.x,
            };
            let height = self.row_height(row);
            (height.min(reach) - LANE_GAP, height.max(reach) + LANE_GAP)
        };
        let mut order: Vec<usize> = (0..self.starts.len()).collect();
        order.sort_by_key(|&wire| (inner_first(wire), wire));
        let mut lanes = vec![0; self.starts.len()];
        for (position, &wire) in order.iter().enumerate() {
            let (low, high) = span(wire);
            lanes[wire] = order[..position]
                .iter()
                .filter(|&&other| {
                    let (other_low, other_high) = span(other);
                    routes[other].side == routes[wire].side && other_low < high && low < other_high
                })
                .map(|&other| lanes[other] + 1)
                .max()
                .unwrap_or(0);
        }
        lanes
    }

    fn detours(&self, routes: &[Route]) -> Vec<usize> {
        (0..self.starts.len())
            .map(|wire| {
                let Some(bypass) = routes[wire].bypass else {
                    return 0;
                };
                let start = self.starts[wire].1;
                let down = self.heading_down(wire, routes[wire]);
                (0..self.starts.len())
                    .filter(|&other| {
                        let position = self.starts[other].1;
                        let inside = if down {
                            (position.x, other) < (start.x, wire)
                        } else {
                            (position.x, other) > (start.x, wire)
                        };
                        other != wire
                            && routes[other].bypass == Some(bypass)
                            && self.heading_down(other, routes[other]) == down
                            && (position.z - start.z).abs() < 2.0 * NEURON_CLEARANCE
                            && inside
                    })
                    .count()
            })
            .collect()
    }

    fn corners(&self, routes: &[Route], neurons: &[Vec3]) -> Vec<Vec<Vec3>> {
        let lanes = self.lanes(routes);
        let detours = self.detours(routes);
        (0..self.starts.len())
            .map(|wire| self.wire_corners(wire, routes[wire], lanes[wire], detours[wire], neurons))
            .collect()
    }

    fn wire_corners(
        &self,
        wire: usize,
        route: Route,
        lane: usize,
        detour: usize,
        neurons: &[Vec3],
    ) -> Vec<Vec3> {
        let bounds = Bounds::of(&self.frame);
        let (row, start) = self.starts[wire];
        let (edge, outward, across) = match route.side {
            Side::Left => (bounds.left, -1.0, 0.0),
            Side::Right => (bounds.right, 1.0, 1.0),
        };
        let end = self.frame.row(row, self.rows).point(across, 0.5);
        let gap = LANE_GAP * (lane + 1) as f32;
        let lane_z = edge + outward * gap;
        let target = match route.around {
            None => {
                let between = (start.z - lane_z) * (start.z - edge) < 0.0
                    && (end.x - start.x).abs() < (lane_z - start.z).abs();
                Vec3::new(end.x, end.y, if between { start.z } else { lane_z })
            }
            Some(around) => {
                let corner_x = match around {
                    Around::Top if start.x > bounds.top => (bounds.top + gap).min(start.x),
                    Around::Top => bounds.top + gap,
                    Around::Bottom if start.x < bounds.bottom => (bounds.bottom - gap).max(start.x),
                    Around::Bottom => bounds.bottom - gap,
                };
                Vec3::new(corner_x, end.y, lane_z)
            }
        };
        let mut points = vec![start];
        if let Some(side) = route.bypass {
            points.extend(
                bypass(start, self.aim(wire, route), side, detour, neurons)
                    .into_iter()
                    .flatten(),
            );
        }
        let from = *points.last().unwrap_or(&start);
        let inside_lanes = (from.z - target.z) * (from.z - edge) < 0.0;
        let shape = if route.around.is_none() && inside_lanes {
            Shape::DiagonalFirst
        } else {
            route.shape
        };
        points.extend(octilinear(from, target, shape, neurons));
        if route.around.is_some() {
            points.push(Vec3::new(end.x, end.y, lane_z));
        }
        points.push(end);
        bends_only(points)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Around {
    Top,
    Bottom,
}

impl Around {
    fn opposite(self) -> Around {
        match self {
            Around::Top => Around::Bottom,
            Around::Bottom => Around::Top,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Route {
    side: Side,
    around: Option<Around>,
    shape: Shape,
    bypass: Option<Side>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    StraightFirst,
    DiagonalFirst,
    DiagonalBetween,
}

struct Bounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl Bounds {
    fn of(frame: &Frame) -> Bounds {
        Bounds {
            left: frame.bottom_left.z,
            right: frame.bottom_left.z + frame.width,
            bottom: frame.bottom_left.x,
            top: frame.bottom_left.x + frame.height,
        }
    }
}

struct Layout {
    key: Vec<f32>,
    routes: Vec<Vec<Route>>,
}

struct Search {
    options: Vec<Vec<Route>>,
    chosen: Vec<usize>,
    pieces: HashMap<usize, (Vec<Vec3>, f32)>,
    pairs: HashMap<(usize, usize), f32>,
}

struct Placement {
    choice: Vec<usize>,
    pieces: Vec<usize>,
    total: f32,
}

fn meter_wire_arcs(meters: &[Meter], neurons: &[Vec3], color: Vec3) -> Vec<Vec<Vec<ArcData>>> {
    meter_wire_corners(meters, neurons)
        .iter()
        .map(|wires| {
            wires
                .iter()
                .map(|corners| rounded(corners, color))
                .collect()
        })
        .collect()
}

fn meter_wire_corners(meters: &[Meter], neurons: &[Vec3]) -> Vec<Vec<Vec<Vec3>>> {
    let key = layout_key(meters, neurons);
    let cached = LAYOUTS.with_borrow(|layouts| {
        layouts
            .iter()
            .find(|layout| {
                layout.key.len() == key.len()
                    && layout
                        .key
                        .iter()
                        .zip(&key)
                        .all(|(a, b)| (a - b).abs() <= LAYOUT_TOLERANCE)
            })
            .map(|layout| layout.routes.clone())
    });
    let routes = cached.unwrap_or_else(|| {
        let routes = chosen_routes(meters, neurons);
        LAYOUTS.with_borrow_mut(|layouts| {
            layouts.insert(
                0,
                Layout {
                    key,
                    routes: routes.clone(),
                },
            );
            layouts.truncate(CACHED_LAYOUTS);
        });
        routes
    });
    meters
        .iter()
        .zip(&routes)
        .map(|(meter, routes)| meter.corners(routes, neurons))
        .collect()
}

fn layout_key(meters: &[Meter], neurons: &[Vec3]) -> Vec<f32> {
    let mut key = vec![meters.len() as f32, neurons.len() as f32];
    for meter in meters {
        let frame = &meter.frame;
        key.extend([
            meter.rows as f32,
            meter.starts.len() as f32,
            frame.bottom_left.x,
            frame.bottom_left.z,
            frame.width,
            frame.height,
        ]);
        for &(row, start) in &meter.starts {
            key.extend([row as f32, start.x, start.z]);
        }
    }
    for neuron in neurons {
        key.extend([neuron.x, neuron.z]);
    }
    key
}

fn chosen_routes(meters: &[Meter], neurons: &[Vec3]) -> Vec<Vec<Route>> {
    let mut searches: Vec<Search> = meters
        .iter()
        .map(|meter| {
            let options: Vec<Vec<Route>> = (0..meter.starts.len())
                .map(|wire| meter.routes(wire, neurons))
                .collect();
            Search {
                chosen: vec![0; options.len()],
                options,
                pieces: HashMap::new(),
                pairs: HashMap::new(),
            }
        })
        .collect();
    let mut paths: Vec<Vec<Vec<Vec3>>> = meters
        .iter()
        .zip(&searches)
        .map(|(meter, search)| meter.corners(&picked(&search.options, &search.chosen), neurons))
        .collect();
    for pass in 0..SETTLE_PASSES {
        let mut changed = false;
        for (index, search) in searches.iter_mut().enumerate() {
            changed |= settle(meters, index, search, &mut paths, neurons, pass > 0);
        }
        if pass > 0 && !changed {
            break;
        }
    }
    searches
        .iter()
        .map(|search| picked(&search.options, &search.chosen))
        .collect()
}

fn picked(options: &[Vec<Route>], choice: &[usize]) -> Vec<Route> {
    options
        .iter()
        .zip(choice)
        .map(|(routes, &option)| routes[option])
        .collect()
}

fn settle(
    meters: &[Meter],
    index: usize,
    search: &mut Search,
    paths: &mut [Vec<Vec<Vec3>>],
    neurons: &[Vec3],
    others_placed: bool,
) -> bool {
    let meter = &meters[index];
    let others: Vec<Frame> = meters
        .iter()
        .enumerate()
        .filter(|&(other, _)| other != index)
        .map(|(_, other)| other.frame)
        .collect();
    let foreign: Vec<&Vec<Vec3>> = paths
        .iter()
        .enumerate()
        .filter(|&(other, _)| others_placed && other != index)
        .flat_map(|(_, wires)| wires)
        .collect();
    let Search {
        options,
        chosen,
        pieces,
        pairs,
    } = search;
    let wires = options.len();
    let most = options.iter().map(Vec::len).max().unwrap_or(0);
    let mut foreign_costs: HashMap<usize, f32> = HashMap::new();
    let mut evaluate = |choice: &[usize]| -> (Vec<usize>, f32) {
        let routes = picked(options, choice);
        let lanes = meter.lanes(&routes);
        let detours = meter.detours(&routes);
        let placed: Vec<usize> = (0..wires)
            .map(|wire| {
                ((wire * most + choice[wire]) * wires + lanes[wire]) * wires + detours[wire]
            })
            .collect();
        let mut total = 0.0;
        for (wire, &piece) in placed.iter().enumerate() {
            let (path, own) = pieces.entry(piece).or_insert_with(|| {
                let path =
                    meter.wire_corners(wire, routes[wire], lanes[wire], detours[wire], neurons);
                let own = path_cost(&path, meter.starts[wire].1, &meter.frame, &others, neurons);
                (path, own)
            });
            total += *own
                + *foreign_costs.entry(piece).or_insert_with(|| {
                    foreign
                        .iter()
                        .map(|other| pair_cost(path, other, FOREIGN_CROSSING_COST))
                        .sum()
                });
        }
        for (wire, &first) in placed.iter().enumerate() {
            for &second in &placed[wire + 1..] {
                total += *pairs.entry((first, second)).or_insert_with(|| {
                    match (pieces.get(&first), pieces.get(&second)) {
                        (Some((first, _)), Some((second, _))) => {
                            pair_cost(first, second, CROSSING_COST)
                        }
                        _ => 0.0,
                    }
                });
            }
        }
        (placed, total)
    };
    let (placed, total) = evaluate(chosen);
    let mut current = Placement {
        choice: chosen.clone(),
        pieces: placed,
        total,
    };
    let mut changed = false;
    for group in 0..=wires {
        let mut best: Option<Placement> = None;
        for choice in trials(group, options, &current.choice) {
            let (placed, total) = evaluate(&choice);
            if best.as_ref().is_none_or(|best| total < best.total - 1e-3) {
                best = Some(Placement {
                    choice,
                    pieces: placed,
                    total,
                });
            }
        }
        if let Some(best) = best.filter(|best| best.total < current.total - 1e-3) {
            current = best;
            changed = true;
        }
    }
    paths[index] = current
        .pieces
        .iter()
        .filter_map(|&piece| pieces.get(&piece).map(|(path, _)| path.clone()))
        .collect();
    *chosen = current.choice;
    changed
}

fn trials(group: usize, options: &[Vec<Route>], chosen: &[usize]) -> Vec<Vec<usize>> {
    let position =
        |wire: usize, route: Route| options[wire].iter().position(|&option| option == route);
    let candidates: Vec<Vec<usize>> = match group.checked_sub(1) {
        None => {
            let mut templates: Vec<Route> = Vec::new();
            for &route in options.iter().flatten() {
                if !templates.contains(&route) {
                    templates.push(route);
                }
            }
            templates
                .iter()
                .map(|&template| {
                    let plain = Route {
                        bypass: None,
                        ..template
                    };
                    chosen
                        .iter()
                        .enumerate()
                        .map(|(wire, &option)| {
                            position(wire, template)
                                .or_else(|| position(wire, plain))
                                .unwrap_or(option)
                        })
                        .collect()
                })
                .collect()
        }
        Some(wire) => (0..options[wire].len())
            .map(|option| {
                let mut trial = chosen.to_vec();
                trial[wire] = option;
                trial
            })
            .collect(),
    };
    let mut trials: Vec<Vec<usize>> = Vec::new();
    for trial in candidates {
        if trial != chosen && !trials.contains(&trial) {
            trials.push(trial);
        }
    }
    trials
}

fn blockers(start: Vec3, aim: f32, neurons: &[Vec3]) -> impl Iterator<Item = Vec3> + '_ {
    let heading = (aim - start.x).signum();
    neurons.iter().copied().filter(move |&neuron| {
        neuron.distance(start) > MIN_SEGMENT_LENGTH
            && (neuron.z - start.z).abs() < NEURON_CLEARANCE
            && (neuron.x - start.x) * heading > 0.0
            && (aim - neuron.x) * heading > 0.0
    })
}

fn bypass(start: Vec3, aim: f32, side: Side, detour: usize, neurons: &[Vec3]) -> Option<[Vec3; 2]> {
    let heading = (aim - start.x).signum();
    let farthest = blockers(start, aim, neurons)
        .map(|neuron| neuron.x * heading)
        .reduce(f32::max)?;
    let offset = NEURON_CLEARANCE + LANE_GAP * detour as f32;
    let sideways = match side {
        Side::Left => -offset,
        Side::Right => offset,
    };
    let step = start + Vec3::new(heading * offset, 0.0, sideways);
    let clear = (farthest + offset).min(aim * heading).max(step.x * heading);
    Some([step, Vec3::new(clear * heading, start.y, step.z)])
}

fn bends_only(mut points: Vec<Vec3>) -> Vec<Vec3> {
    points.dedup_by(|a, b| a.distance(*b) < MIN_SEGMENT_LENGTH);
    let mut bends = points[..1].to_vec();
    for pair in points.windows(2).skip(1) {
        let before = bends[bends.len() - 1];
        let (along, onward) = (flat(pair[0] - before), flat(pair[1] - pair[0]));
        if along.perp_dot(onward).abs() > 1e-6 * along.length() * onward.length()
            || along.dot(onward) < 0.0
        {
            bends.push(pair[0]);
        }
    }
    bends.extend(points.last());
    bends
}

fn octilinear(from: Vec3, to: Vec3, shape: Shape, neurons: &[Vec3]) -> Vec<Vec3> {
    let delta = to - from;
    let diagonal = delta.x.abs().min(delta.z.abs());
    let slant = Vec3::new(
        delta.x.signum() * diagonal,
        0.0,
        delta.z.signum() * diagonal,
    );
    let straight = delta - slant;
    match shape {
        Shape::StraightFirst => vec![from + straight, to],
        Shape::DiagonalFirst => vec![from + slant, to],
        Shape::DiagonalBetween => {
            let near = near_neurons(&[from, to], from, neurons);
            DIAGONAL_SHARES
                .iter()
                .map(|share| {
                    let bend = from + straight * *share;
                    vec![bend, bend + slant, to]
                })
                .min_by(|a, b| {
                    let cost = |path: &[Vec3]| over_neurons(&[&[from], path].concat(), &near);
                    cost(a).total_cmp(&cost(b))
                })
                .unwrap_or_default()
        }
    }
}

fn path_cost(path: &[Vec3], start: Vec3, own: &Frame, others: &[Frame], neurons: &[Vec3]) -> f32 {
    let length: f32 = path.windows(2).map(|pair| pair[0].distance(pair[1])).sum();
    let bounds = Bounds::of(own);
    let center = (bounds.left + bounds.right) / 2.0;
    let end = path.last().map_or(center, |end| end.z);
    let far_side = (start.z < center) != (end < center);
    METER_COST * over_meters(path, own, others)
        + NEURON_COST * over_neurons(path, &near_neurons(path, start, neurons))
        + length
        + if far_side { FAR_SIDE_COST } else { 0.0 }
}

fn over_meters(path: &[Vec3], own: &Frame, others: &[Frame]) -> f32 {
    path.windows(2)
        .map(|pair| {
            inside_length(pair[0], pair[1], own, -1e-3)
                + others
                    .iter()
                    .map(|other| inside_length(pair[0], pair[1], other, CLEARANCE))
                    .sum::<f32>()
        })
        .sum()
}

fn inside_length(a: Vec3, b: Vec3, frame: &Frame, margin: f32) -> f32 {
    let bounds = Bounds::of(frame);
    let mut enter = 0.0_f32;
    let mut leave = 1.0_f32;
    for (start, delta, low, high) in [
        (a.x, b.x - a.x, bounds.bottom - margin, bounds.top + margin),
        (a.z, b.z - a.z, bounds.left - margin, bounds.right + margin),
    ] {
        if delta.abs() < f32::EPSILON {
            if start <= low || start >= high {
                return 0.0;
            }
        } else {
            let first = (low - start) / delta;
            let second = (high - start) / delta;
            enter = enter.max(first.min(second));
            leave = leave.min(first.max(second));
        }
    }
    (leave - enter).max(0.0) * a.distance(b)
}

fn near_neurons(path: &[Vec3], start: Vec3, neurons: &[Vec3]) -> Vec<Vec3> {
    let (low, high) = flat_bounds(path);
    neurons
        .iter()
        .copied()
        .filter(|&neuron| {
            flat(neuron).cmpgt(low - NEURON_CLEARANCE).all()
                && flat(neuron).cmplt(high + NEURON_CLEARANCE).all()
                && neuron.distance(start) > MIN_SEGMENT_LENGTH
        })
        .collect()
}

fn over_neurons(path: &[Vec3], neurons: &[Vec3]) -> f32 {
    path.windows(2)
        .map(|pair| {
            neurons
                .iter()
                .map(|&neuron| near_length(pair[0], pair[1], neuron))
                .sum::<f32>()
        })
        .sum()
}

fn flat(point: Vec3) -> Vec2 {
    Vec2::new(point.x, point.z)
}

fn near_length(a: Vec3, b: Vec3, center: Vec3) -> f32 {
    let (a, b, center) = (flat(a), flat(b), flat(center));
    let length = a.distance(b);
    if length < f32::EPSILON {
        return 0.0;
    }
    let direction = (b - a) / length;
    let along = (center - a).dot(direction);
    let off = (center - a).perp_dot(direction).abs();
    if off >= NEURON_CLEARANCE {
        return 0.0;
    }
    let half = (NEURON_CLEARANCE * NEURON_CLEARANCE - off * off).sqrt();
    ((along + half).min(length) - (along - half).max(0.0)).max(0.0)
}

fn visible_segments(path: &[Vec3]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    let mut passed = 0.0;
    path.windows(2).filter_map(move |pair| {
        let (a, b) = (flat(pair[0]), flat(pair[1]));
        let length = a.distance(b);
        let hidden = (NODE_RADIUS - passed).max(0.0);
        passed += length;
        (length > hidden).then(|| (a.lerp(b, hidden / length), b))
    })
}

fn flat_bounds(path: &[Vec3]) -> (Vec2, Vec2) {
    path.iter().fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(low, high), &point| (low.min(flat(point)), high.max(flat(point))),
    )
}

fn pair_cost(a: &[Vec3], b: &[Vec3], crossing_cost: f32) -> f32 {
    let ((a_low, a_high), (b_low, b_high)) = (flat_bounds(a), flat_bounds(b));
    if a_high.cmplt(b_low - OVERLAP_DISTANCE).any() || b_high.cmplt(a_low - OVERLAP_DISTANCE).any()
    {
        return 0.0;
    }
    let mut cost = 0.0;
    for (a0, a1) in visible_segments(a) {
        for (b0, b1) in visible_segments(b) {
            if a0.max(a1).cmplt(b0.min(b1) - OVERLAP_DISTANCE).any()
                || b0.max(b1).cmplt(a0.min(a1) - OVERLAP_DISTANCE).any()
            {
                continue;
            }
            let (along, across) = (a1 - a0, b1 - b0);
            let turn = along.perp_dot(across);
            if turn.abs() < 1e-6 * along.length() * across.length() {
                cost += OVERLAP_COST * overlap(a0, a1, b0, b1);
            } else {
                let t = (b0 - a0).perp_dot(across) / turn;
                let u = (b0 - a0).perp_dot(along) / turn;
                if (0.0..1.0).contains(&t) && (0.0..1.0).contains(&u) {
                    cost += crossing_cost;
                }
            }
        }
    }
    cost
}

fn overlap(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> f32 {
    let length = a0.distance(a1);
    if length < f32::EPSILON {
        return 0.0;
    }
    let direction = (a1 - a0) / length;
    if (b0 - a0).perp_dot(direction).abs() >= OVERLAP_DISTANCE {
        return 0.0;
    }
    let (first, second) = ((b0 - a0).dot(direction), (b1 - a0).dot(direction));
    (first.max(second).min(length) - first.min(second).max(0.0)).max(0.0)
}

fn rounded(corners: &[Vec3], color: Vec3) -> Vec<ArcData> {
    let mut arcs = Vec::new();
    let mut from = corners[0];
    for corner in corners.windows(3) {
        let [before, at, after] = [corner[0], corner[1], corner[2]];
        let reach = BEND_RADIUS
            .min(at.distance(before) / 2.0)
            .min(at.distance(after) / 2.0);
        let entry = at + (before - at).normalize_or_zero() * reach;
        let exit = at + (after - at).normalize_or_zero() * reach;
        arcs.push(straight(from, entry, color));
        arcs.push(ArcData {
            turn: (at - before)
                .cross(after - at)
                .y
                .atan2((at - before).dot(after - at)),
            ..straight(entry, exit, color)
        });
        from = exit;
    }
    arcs.push(straight(from, corners[corners.len() - 1], color));
    arcs.retain(|arc| arc.start.distance(arc.end) > MIN_SEGMENT_LENGTH);
    arcs
}

fn straight(start: Vec3, end: Vec3, color: Vec3) -> ArcData {
    ArcData {
        start,
        turn: 0.0,
        end,
        color,
        _padding: Default::default(),
    }
}

pub fn arc_points(arc: &ArcData) -> Vec<Vec3> {
    let chord = arc.end - arc.start;
    if arc.turn.abs() < STRAIGHT_TURN || chord.length() < MIN_SEGMENT_LENGTH {
        return vec![arc.start, arc.end];
    }
    let radius = 0.5 * chord.length() / (0.5 * arc.turn.abs()).sin();
    let across = Vec3::Y.cross(chord.normalize()) * arc.turn.signum();
    let center = arc.start.midpoint(arc.end) + across * radius * (0.5 * arc.turn).cos();
    (0..=ARC_POINTS)
        .map(|step| {
            let angle = arc.turn * step as f32 / ARC_POINTS as f32;
            center + Quat::from_rotation_y(angle) * (arc.start - center)
        })
        .collect()
}

pub fn connection_being_drawn(points: &[Vec3], palette: &Palette) -> Vec<ConnectionData> {
    let last = points.len().saturating_sub(2);
    points
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[0].distance(pair[1]) > MIN_SEGMENT_LENGTH)
        .map(|(index, pair)| ConnectionData {
            position_a: pair[0],
            position_b: pair[1],
            strength: 1.0,
            directional: if index == last { 1.0 } else { 0.0 },
            start_color: palette.neutral,
            end_color: palette.neutral,
            _padding: Default::default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_spike_detector, spawn_spike_detector_wire, spawn_voltmeter};
    use crate::rendering::colors::Theme;
    use hecs::World;

    fn frame() -> Frame {
        Frame::new(
            Vec3::ZERO,
            &MeterSize {
                width: 8.0,
                height: 6.0,
            },
        )
    }

    fn meter(frame: Frame, rows: usize, starts: &[(usize, Vec3)]) -> Meter {
        Meter {
            frame,
            rows,
            starts: starts.to_vec(),
        }
    }

    fn stacked(frame: Frame, heights: &[f32], z: f32) -> Meter {
        let starts: Vec<(usize, Vec3)> = heights
            .iter()
            .enumerate()
            .map(|(row, &height)| (row, Vec3::new(height, 0.0, z)))
            .collect();
        meter(frame, heights.len(), &starts)
    }

    fn starts(meters: &[Meter]) -> Vec<Vec3> {
        meters
            .iter()
            .flat_map(|meter| meter.starts.iter().map(|&(_, start)| start))
            .collect()
    }

    fn meter_wire_paths(meters: &[Meter], neurons: &[Vec3]) -> Vec<Vec<Vec<Vec3>>> {
        meter_wire_arcs(meters, neurons, Vec3::ONE)
            .iter()
            .map(|wires| {
                wires
                    .iter()
                    .map(|arcs| {
                        let mut points: Vec<Vec3> = arcs.iter().flat_map(arc_points).collect();
                        points.dedup_by(|point, kept| point.distance(*kept) <= MIN_SEGMENT_LENGTH);
                        points
                    })
                    .collect()
            })
            .collect()
    }

    fn paths(meters: &[Meter]) -> Vec<Vec<Vec<Vec3>>> {
        meter_wire_paths(meters, &starts(meters))
    }

    fn passes_over(frame: &Frame, path: &[Vec3]) -> bool {
        comes_within(frame, path, -1e-3)
    }

    fn comes_within(frame: &Frame, path: &[Vec3], margin: f32) -> bool {
        let bounds = Bounds::of(frame);
        let inside = |point: Vec3| {
            point.x > bounds.bottom - margin
                && point.x < bounds.top + margin
                && point.z > bounds.left - margin
                && point.z < bounds.right + margin
        };
        path.windows(2)
            .any(|pair| (0..=50).any(|step| inside(pair[0].lerp(pair[1], step as f32 / 50.0))))
    }

    fn point_distance(point: Vec2, (a, b): (Vec2, Vec2)) -> f32 {
        let along = b - a;
        let t = ((point - a).dot(along) / along.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
        point.distance(a + along * t)
    }

    fn distance(first: &[Vec3], second: &[Vec3]) -> f32 {
        let mut closest = f32::INFINITY;
        for (a0, a1) in visible_segments(first) {
            for (b0, b1) in visible_segments(second) {
                let (along, across) = (a1 - a0, b1 - b0);
                let turn = along.perp_dot(across);
                if turn.abs() > f32::EPSILON {
                    let t = (b0 - a0).perp_dot(across) / turn;
                    let u = (b0 - a0).perp_dot(along) / turn;
                    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                        return 0.0;
                    }
                }
                closest = closest
                    .min(point_distance(a0, (b0, b1)))
                    .min(point_distance(a1, (b0, b1)))
                    .min(point_distance(b0, (a0, a1)))
                    .min(point_distance(b1, (a0, a1)));
            }
        }
        closest
    }

    fn assert_apart(paths: &[Vec<Vec3>]) {
        for (index, first) in paths.iter().enumerate() {
            for (other, second) in paths.iter().enumerate().skip(index + 1) {
                let distance = distance(first, second);
                assert!(
                    distance > 0.9 * LANE_GAP,
                    "wires {index} and {other} come {distance} apart\n{first:?}\n{second:?}"
                );
            }
        }
    }

    fn assert_off_neurons(path: &[Vec3], neurons: &[Vec3]) {
        for &neuron in neurons.iter().filter(|&&neuron| neuron != path[0]) {
            let distance = visible_segments(path)
                .map(|segment| point_distance(flat(neuron), segment))
                .fold(f32::INFINITY, f32::min);
            assert!(
                distance > NODE_RADIUS + 0.2,
                "passes {distance} from {neuron}\n{path:?}"
            );
        }
    }

    const NEURONS: [Vec3; 8] = [
        Vec3::new(10.0, 0.0, 6.0),
        Vec3::new(5.0, 0.0, 1.0),
        Vec3::new(3.2, 0.0, 7.9),
        Vec3::new(-8.0, 0.0, 4.5),
        Vec3::new(-3.3, 0.0, 0.2),
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(9.0, 0.0, 14.0),
        Vec3::new(-9.0, 0.0, -2.0),
    ];

    #[test]
    fn a_wire_enters_its_own_row_from_the_side_facing_the_neuron_without_passing_over_the_meter() {
        let frame = frame();
        let rows = 3;
        for neuron in NEURONS {
            for row in 0..rows {
                let path = &paths(&[meter(frame, rows, &[(row, neuron)])])[0][0];
                let facing = if neuron.z < 4.0 { 0.0 } else { 1.0 };
                assert_eq!(path[0], neuron);
                assert_eq!(
                    *path.last().unwrap(),
                    frame.row(row, rows).point(facing, 0.5)
                );
                assert!(!passes_over(&frame, path), "{neuron} to row {row}");
            }
        }
    }

    #[test]
    fn every_wire_runs_level_upright_or_at_45_degrees_and_enters_its_row_level() {
        let frame = frame();
        let rows = 3;
        let mut meters: Vec<Meter> = NEURONS
            .iter()
            .flat_map(|&neuron| (0..rows).map(move |row| meter(frame, rows, &[(row, neuron)])))
            .collect();
        meters.push(stacked(frame, &[12.0, 9.0, 6.0], 3.0));
        meters.push(stacked(frame, &[5.0, 1.0, -4.0], -0.4));
        for meter in meters {
            let neurons = starts(std::slice::from_ref(&meter));
            for corners in &meter_wire_corners(&[meter], &neurons)[0] {
                for pair in corners.windows(2) {
                    let (up, right) =
                        ((pair[1].x - pair[0].x).abs(), (pair[1].z - pair[0].z).abs());
                    assert!(
                        up < 1e-4 || right < 1e-4 || (up - right).abs() < 1e-4,
                        "{corners:?}"
                    );
                }
                let last = &corners[corners.len() - 2..];
                assert!((last[1].x - last[0].x).abs() < 1e-4, "{corners:?}");
            }
        }
    }

    #[test]
    fn wires_of_one_meter_keep_a_lane_apart_and_off_the_other_neurons() {
        let frame = frame();
        for meter in [
            stacked(frame, &[12.0, 9.0, 6.0], 3.0),
            stacked(frame, &[12.0, 9.0, 6.0], -3.0),
            stacked(frame, &[12.0, 9.0, 6.0], -0.4),
            stacked(frame, &[5.0, 1.0, -4.0], -6.0),
            stacked(frame, &[2.0, 0.0, -2.0], 12.0),
        ] {
            let neurons = starts(std::slice::from_ref(&meter));
            let paths = &meter_wire_paths(std::slice::from_ref(&meter), &neurons)[0];
            assert_apart(paths);
            for path in paths {
                assert!(!passes_over(&frame, path));
                assert_off_neurons(path, &neurons);
            }
        }
    }

    #[test]
    fn wires_of_two_meters_below_their_layers_keep_apart() {
        let input = [8.8, 3.68, -1.44].map(|x| Vec3::new(x, 0.0, -14.08));
        let output = [8.8, 3.68, -1.44].map(|x| Vec3::new(x, 0.0, 2.56));
        let left = Frame {
            bottom_left: Vec3::new(-17.6, 0.0, -13.64),
            width: 15.4,
            height: 9.1,
        };
        let right = Frame {
            bottom_left: Vec3::new(-18.3, 0.0, 6.74),
            width: 15.4,
            height: 11.5,
        };
        let meters = [
            stacked(left, &input.map(|neuron| neuron.x), input[0].z),
            stacked(right, &output.map(|neuron| neuron.x), output[0].z),
        ];
        let mut neurons = starts(&meters);
        neurons.extend([
            Vec3::new(6.24, 0.0, -6.4),
            Vec3::new(1.12, 0.0, -5.12),
            Vec3::new(3.68, 0.0, -30.72),
        ]);
        let paths = meter_wire_paths(&meters, &neurons);
        let all: Vec<Vec<Vec3>> = paths.iter().flatten().cloned().collect();
        assert_apart(&all);
        for path in &all {
            assert_off_neurons(path, &neurons);
            assert!(!passes_over(&left, path) && !passes_over(&right, path));
        }
    }

    #[test]
    fn a_wire_goes_around_a_meter_standing_between_its_neuron_and_its_row() {
        let frame = frame();
        let between = Frame::new(
            Vec3::new(0.0, 0.0, -9.2),
            &MeterSize {
                width: 8.0,
                height: 6.0,
            },
        );
        let neuron = Vec3::new(1.0, 0.0, -14.0);
        let paths = paths(&[meter(frame, 1, &[(0, neuron)]), meter(between, 1, &[])]);
        let path = &paths[0][0];
        assert_eq!(*path.last().unwrap(), frame.row(0, 1).point(0.0, 0.5));
        assert!(!comes_within(&between, path, 0.5 * LANE_GAP));
        assert!(!passes_over(&frame, path));
    }

    #[test]
    fn wires_from_stacked_neurons_stay_off_a_meter_close_beside_theirs() {
        let frame = frame();
        let beside = Frame::new(
            Vec3::new(0.0, 0.0, -9.2),
            &MeterSize {
                width: 8.0,
                height: 6.0,
            },
        );
        let paths = paths(&[
            stacked(frame, &[12.0, 9.0, 6.0], 3.0),
            meter(beside, 1, &[]),
        ]);
        for (row, path) in paths[0].iter().enumerate() {
            let end = *path.last().unwrap();
            assert!(
                end == frame.row(row, 3).point(0.0, 0.5)
                    || end == frame.row(row, 3).point(1.0, 0.5)
            );
            assert!(!comes_within(&beside, path, 0.5 * LANE_GAP), "row {row}");
            assert!(!passes_over(&frame, path), "row {row}");
        }
    }

    #[test]
    fn pieces_of_a_wire_meet_end_to_end_in_the_same_direction() {
        let heading = |arc: &ArcData, share: f32| {
            Quat::from_rotation_y(share * arc.turn) * (arc.end - arc.start).normalize()
        };
        let frame = frame();
        let mut meters: Vec<Meter> = NEURONS
            .iter()
            .flat_map(|&neuron| (0..3).map(move |row| meter(frame, 3, &[(row, neuron)])))
            .collect();
        meters.push(stacked(frame, &[12.0, 9.0, 6.0], 3.0));
        meters.push(stacked(frame, &[5.0, 1.0, -4.0], -0.4));
        let mut bends = 0;
        for meter in meters {
            let neurons = starts(std::slice::from_ref(&meter));
            for arcs in &meter_wire_arcs(&[meter], &neurons, Vec3::ONE)[0] {
                for arc in arcs {
                    let points = arc_points(arc);
                    assert!(points[0].distance(arc.start) < 1e-4);
                    assert!(points[points.len() - 1].distance(arc.end) < 1e-4, "{arc:?}");
                    bends += usize::from(arc.turn != 0.0);
                }
                for pair in arcs.windows(2) {
                    assert_eq!(pair[0].end, pair[1].start);
                    let (out, into) = (heading(&pair[0], 0.5), heading(&pair[1], -0.5));
                    assert!(out.distance(into) < 1e-4, "{:?}", pair);
                }
            }
        }
        assert!(bends > 50, "{bends} bends");
    }

    #[test]
    fn spike_detector_wires_follow_the_connection_view_but_voltmeter_wires_stay() {
        let mut world = World::new();
        let neurons = [0.0, 3.0].map(|z| {
            world.spawn((
                Position {
                    position: Vec3::new(0.0, 0.0, z),
                },
                NodeState::default(),
            ))
        });
        let detector = spawn_spike_detector(&mut world, neurons[0], Vec3::X * 10.0);
        spawn_spike_detector_wire(&mut world, neurons[1], detector);
        spawn_voltmeter(&mut world, neurons[1], Vec3::X * 20.0);
        let starts = neurons.map(|neuron| world.get::<&Position>(neuron).unwrap().position);
        let mut wires = |view: ConnectionView, selected| {
            view.set_in(&mut world);
            Selected::set(&mut world, selected);
            collect_meter_wires(&world, &Theme::default().palette())
                .iter()
                .filter(|segment| {
                    starts
                        .iter()
                        .any(|start| segment.start.distance(*start) < MIN_SEGMENT_LENGTH)
                })
                .count()
        };
        assert_eq!(wires(ConnectionView::All, None), 3);
        assert_eq!(wires(ConnectionView::LastSelected, Some(neurons[0])), 2);
        assert_eq!(wires(ConnectionView::LastSelected, Some(detector)), 3);
        assert_eq!(wires(ConnectionView::LastSelected, None), 1);
        assert_eq!(wires(ConnectionView::None, Some(detector)), 1);
    }
}
