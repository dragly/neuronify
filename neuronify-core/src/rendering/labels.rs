use std::collections::{BTreeMap, HashMap};

use glam::{Vec2, Vec3, Vec4};
use visula::{
    text::{GlyphAtlas, TessellatedGeometry},
    Expression, GlyphDelegate, GlyphVertex, Glyphs,
};

use crate::components::{Cable, Connection, Label, MeterSize, NoteSize, Position};
use crate::constants::{CONNECTION_WIDTH_PER_STRENGTH, NODE_RADIUS};
use crate::measurement::firing_rate::RatePlot;
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::visual_input::{visual_input_frame, VisualInput};
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::colors::{srgb, Theme};
use crate::rendering::gpu_types::ArcData;
use crate::rendering::notes::{note_plane, shifted, translated, NOTE_FONT_DATA, NOTE_TEXT_Y};
use crate::rendering::paths::ConnectionPath;
use crate::rendering::selection::element_radius;
use hecs::Entity;

pub const LABEL_HEIGHT: f32 = 0.8;
pub const LABEL_GAP: f32 = 0.3;
const TITLE_HEIGHT: f32 = 0.55;
const TITLE_INSET: f32 = 0.25;
const SINGLE_LINE: f32 = 1000.0;
const CLEARANCE: f32 = 0.1;
const OBSTACLE_COST: usize = 4;
const LABEL_COST: usize = 40;
const INHIBITORY_BAR_SCALE: f32 = 1.2;
pub const EMPTY_METER_HINT: &str = "Connect a neuron";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Placement {
    Beside(Side),
    TopLeft,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Side {
    Above,
    Below,
    Right,
    Left,
}

const SIDES: [Side; 4] = [Side::Above, Side::Below, Side::Right, Side::Left];

impl Side {
    fn area(self, center: Vec2, size: Vec2) -> Area {
        let gap = NODE_RADIUS + LABEL_GAP;
        let corner = match self {
            Side::Above => Vec2::new(-0.5 * size.x, gap),
            Side::Below => Vec2::new(-0.5 * size.x, -gap - size.y),
            Side::Right => Vec2::new(gap, -0.5 * size.y),
            Side::Left => Vec2::new(-gap - size.x, -0.5 * size.y),
        };
        Area {
            low: center + corner,
            high: center + corner + size,
        }
    }
}

pub struct Drawn<'a> {
    pub radii: &'a HashMap<Entity, f32>,
    pub paths: &'a BTreeMap<Entity, ConnectionPath>,
    pub wires: &'a [ArcData],
}

fn on_screen(point: Vec3) -> Vec2 {
    Vec2::new(point.z, point.x)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Area {
    low: Vec2,
    high: Vec2,
}

impl Area {
    fn of(frame: &Frame) -> Area {
        let low = on_screen(frame.bottom_left);
        Area {
            low,
            high: low + Vec2::new(frame.width, frame.height),
        }
    }

    fn around(center: Vec2, half: Vec2) -> Area {
        Area {
            low: center - half,
            high: center + half,
        }
    }

    fn grown(self, by: f32) -> Area {
        Area {
            low: self.low - by,
            high: self.high + by,
        }
    }

    fn overlaps(&self, other: &Area) -> bool {
        self.low.cmplt(other.high).all() && other.low.cmplt(self.high).all()
    }

    fn crossed_by(&self, start: Vec2, end: Vec2) -> bool {
        let along = end - start;
        let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
        for axis in 0..2 {
            if along[axis] == 0.0 {
                if start[axis] < self.low[axis] || start[axis] > self.high[axis] {
                    return false;
                }
                continue;
            }
            let low = (self.low[axis] - start[axis]) / along[axis];
            let high = (self.high[axis] - start[axis]) / along[axis];
            enter = enter.max(low.min(high));
            leave = leave.min(low.max(high));
        }
        enter <= leave
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Stroke {
    start: Vec2,
    end: Vec2,
    radius: f32,
}

impl Stroke {
    fn between(start: Vec3, end: Vec3, radius: f32) -> Stroke {
        Stroke {
            start: on_screen(start),
            end: on_screen(end),
            radius,
        }
    }

    fn dot(center: Vec3, radius: f32) -> Stroke {
        Stroke::between(center, center, radius)
    }

    fn bounds(&self) -> Area {
        Area {
            low: self.start.min(self.end),
            high: self.start.max(self.end),
        }
        .grown(self.radius)
    }

    fn crosses(&self, area: &Area) -> bool {
        area.grown(self.radius).crossed_by(self.start, self.end)
    }
}

#[derive(Default)]
struct Obstacles {
    strokes: Vec<Stroke>,
    areas: Vec<Area>,
}

impl Obstacles {
    fn of(world: &hecs::World, drawn: &Drawn) -> Obstacles {
        let mut obstacles = Obstacles::default();
        for path in drawn.paths.values() {
            obstacles.strokes.extend(
                path.points
                    .windows(2)
                    .map(|pair| Stroke::between(pair[0], pair[1], path.radius)),
            );
            if let (true, Some(&end)) = (path.directional, path.points.last()) {
                let radius = if path.inhibitory {
                    INHIBITORY_BAR_SCALE * path.terminal_radius
                } else {
                    path.terminal_radius
                };
                obstacles.strokes.push(Stroke::dot(end, radius));
            }
        }
        for (connection, _) in world.query::<(&Connection, &Cable)>().iter() {
            let (Ok(from), Ok(to)) = (
                world.get::<&Position>(connection.from),
                world.get::<&Position>(connection.to),
            ) else {
                continue;
            };
            let radius = [connection.from, connection.to]
                .iter()
                .filter_map(|end| drawn.radii.get(end))
                .fold(0.0_f32, |widest, &radius| widest.max(radius));
            obstacles
                .strokes
                .push(Stroke::between(from.position, to.position, radius));
        }
        obstacles.strokes.extend(drawn.wires.iter().map(|wire| {
            Stroke::between(wire.start, wire.end, 0.5 * CONNECTION_WIDTH_PER_STRENGTH)
        }));
        for (entity, position) in world.query::<(Entity, &Position)>().iter() {
            let position = position.position;
            if let Ok(size) = world.get::<&MeterSize>(entity) {
                obstacles.areas.push(Area::of(&Frame::new(position, &size)));
            } else if let Ok(size) = world.get::<&NoteSize>(entity) {
                obstacles.areas.push(Area::around(
                    on_screen(position),
                    0.5 * Vec2::new(size.width, size.height),
                ));
            } else if world.satisfies::<&VisualInput>(entity) {
                obstacles
                    .areas
                    .push(Area::of(&visual_input_frame(position)));
            } else {
                obstacles.strokes.push(Stroke::dot(
                    position,
                    element_radius(world, entity, drawn.radii),
                ));
            }
        }
        obstacles
    }

    fn near(&self, reach: &Area) -> Obstacles {
        Obstacles {
            strokes: self
                .strokes
                .iter()
                .filter(|stroke| stroke.bounds().overlaps(reach))
                .copied()
                .collect(),
            areas: self
                .areas
                .iter()
                .filter(|area| area.overlaps(reach))
                .copied()
                .collect(),
        }
    }

    fn blocking(&self, area: &Area) -> usize {
        let strokes = self.strokes.iter().filter(|stroke| stroke.crosses(area));
        let areas = self.areas.iter().filter(|other| other.overlaps(area));
        strokes.count() + areas.count()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Measure {
    line: Vec2,
    ink: Area,
}

impl Measure {
    fn ink_on(&self, side: Side, center: Vec2) -> Area {
        let line = side.area(center, self.line);
        Area {
            low: line.low + self.ink.low,
            high: line.low + self.ink.high,
        }
    }
}

struct Candidate {
    center: Vec2,
    measure: Measure,
    reach: Area,
    near: Obstacles,
}

impl Candidate {
    fn new(center: Vec2, measure: Measure, obstacles: &Obstacles) -> Candidate {
        let reach = Area::around(
            center,
            Vec2::splat(NODE_RADIUS + LABEL_GAP + measure.line.max_element() + CLEARANCE),
        );
        Candidate {
            center,
            measure,
            reach,
            near: obstacles.near(&reach),
        }
    }

    fn ink(&self, side: Side) -> Area {
        self.measure.ink_on(side, self.center)
    }

    fn cost(&self, side: Side, placed: &[Area], spare: &[Area]) -> usize {
        let ink = self.ink(side).grown(CLEARANCE);
        let covered = |areas: &[Area]| areas.iter().filter(|area| area.overlaps(&ink)).count();
        OBSTACLE_COST * self.near.blocking(&ink) + LABEL_COST * covered(placed) + covered(spare)
    }

    fn cheapest_side(&self, placed: &[Area], spare: &[Area]) -> Side {
        let costs = SIDES.map(|side| (self.cost(side, placed, spare), side));
        costs
            .into_iter()
            .fold((usize::MAX, Side::Above), |cheapest, cost| {
                if cost.0 < cheapest.0 {
                    cost
                } else {
                    cheapest
                }
            })
            .1
    }
}

fn free_sides(candidates: &[Candidate]) -> Vec<Side> {
    let mut placed = Vec::new();
    let mut sides = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let spare: Vec<Area> = candidates
            .iter()
            .enumerate()
            .filter(|(other, neighbour)| {
                *other > index && neighbour.reach.overlaps(&candidate.reach)
            })
            .flat_map(|(_, neighbour)| SIDES.map(|side| neighbour.ink(side)))
            .collect();
        let side = candidate.cheapest_side(&placed, &spare);
        placed.push(candidate.ink(side));
        sides.push(side);
    }
    sides
}

fn text_color(theme: Theme) -> Vec4 {
    match theme {
        Theme::Dark => srgb(205, 214, 244).extend(1.0),
        Theme::Light => srgb(76, 79, 105).extend(1.0),
    }
}

pub struct LabelRenderer {
    theme: Theme,
    text: Glyphs,
    atlas: GlyphAtlas,
    laid: HashMap<(String, Placement), TessellatedGeometry<GlyphVertex>>,
    measures: HashMap<String, Measure>,
    uploaded: Vec<(String, Placement, Vec3)>,
}

impl LabelRenderer {
    pub fn new(
        application: &visula::Application,
        theme: Theme,
    ) -> Result<Self, visula::error::Error> {
        Ok(Self {
            theme,
            text: glyphs(application, theme)?,
            atlas: GlyphAtlas::new(NOTE_FONT_DATA)?,
            laid: HashMap::new(),
            measures: HashMap::new(),
            uploaded: Vec::new(),
        })
    }

    pub fn update(
        &mut self,
        application: &visula::Application,
        world: &hecs::World,
        theme: Theme,
        drawn: &Drawn,
    ) {
        if theme != self.theme {
            match glyphs(application, theme) {
                Ok(text) => self.text = text,
                Err(error) => log::error!("Kept the label colour of the last theme: {error}"),
            }
            self.theme = theme;
            self.uploaded.clear();
        }
        let (atlas, measures) = (&mut self.atlas, &mut self.measures);
        let placed = placed_labels(world, drawn, |text| match measures.get(text) {
            Some(&measure) => measure,
            None => {
                let measure = measure_label(atlas, text);
                measures.insert(text.to_string(), measure);
                measure
            }
        });
        self.measures
            .retain(|text, _| placed.iter().any(|(placed, ..)| placed == text));
        if placed == self.uploaded {
            return;
        }
        let mut geometry = TessellatedGeometry::default();
        for (text, placement, anchor) in &placed {
            let key = (text.clone(), *placement);
            let laid = self
                .laid
                .entry(key)
                .or_insert_with(|| lay_out(&mut self.atlas, text, *placement));
            geometry.merge(&translated(laid, *anchor));
        }
        self.text
            .update_atlas(&application.device, &application.queue, &self.atlas);
        self.text.update_geometry(
            &application.device,
            &application.queue,
            &geometry.vertices,
            &geometry.indices,
        );
        self.laid.retain(|(text, placement), _| {
            placed
                .iter()
                .any(|(placed, kind, _)| placed == text && kind == placement)
        });
        self.uploaded = placed;
    }

    pub fn render(&self, data: &mut visula::RenderData) {
        use visula::Renderable;
        self.text.render(data);
    }
}

fn glyphs(
    application: &visula::Application,
    theme: Theme,
) -> Result<Glyphs, visula_core::ShaderError> {
    Glyphs::new(
        &application.rendering_descriptor(),
        &GlyphDelegate {
            color: text_color(theme).into(),
            transform: Expression::from(note_plane(NOTE_TEXT_Y)),
        },
    )
}

fn measure_label(atlas: &mut GlyphAtlas, text: &str) -> Measure {
    let wrapped = match atlas.lay_out_wrapped(text, LABEL_HEIGHT, SINGLE_LINE) {
        Ok(wrapped) => wrapped,
        Err(e) => {
            log::error!("Failed to measure a label: {e:?}");
            return Measure::default();
        }
    };
    let line = Vec2::new(wrapped.width, wrapped.height);
    let corners = wrapped
        .geometry
        .vertices
        .iter()
        .map(|vertex| Vec2::from(vertex.position) + Vec2::Y * line.y);
    let (low, high) = corners.fold((Vec2::MAX, Vec2::MIN), |(low, high), corner| {
        (low.min(corner), high.max(corner))
    });
    Measure {
        line,
        ink: Area { low, high },
    }
}

fn placed_labels(
    world: &hecs::World,
    drawn: &Drawn,
    mut measure: impl FnMut(&str) -> Measure,
) -> Vec<(String, Placement, Vec3)> {
    let labelled: Vec<(String, Vec3)> = world
        .query::<(&Label, &Position)>()
        .iter()
        .filter(|(label, _)| !label.text.is_empty())
        .map(|(label, position)| (label.text.clone(), position.position))
        .collect();
    let obstacles = if labelled.is_empty() {
        Obstacles::default()
    } else {
        Obstacles::of(world, drawn)
    };
    let candidates: Vec<Candidate> = labelled
        .iter()
        .map(|(text, position)| Candidate::new(on_screen(*position), measure(text), &obstacles))
        .collect();
    let mut placed: Vec<(String, Placement, Vec3)> = labelled
        .into_iter()
        .zip(free_sides(&candidates))
        .map(|((text, position), side)| (text, Placement::Beside(side), position))
        .collect();
    for (_, (plot, position, size)) in world
        .query::<(Entity, (&RatePlot, &Position, &MeterSize))>()
        .iter()
    {
        let corner = Frame::new(position.position, size).corner(Corner::TopLeft);
        placed.push((
            format!("{:.0} Hz", plot.maximum.0),
            Placement::TopLeft,
            corner,
        ));
    }
    for meter in world
        .query::<Entity>()
        .with::<&MeterSize>()
        .without::<&RatePlot>()
        .iter()
    {
        let Some(frame) = Frame::of(world, meter) else {
            continue;
        };
        let measured = measured(world, meter);
        for (row, (_, target)) in measured.iter().enumerate() {
            let Ok(label) = world.get::<&Label>(*target) else {
                continue;
            };
            if label.text.is_empty() {
                continue;
            }
            let corner = frame.row(row, measured.len()).corner(Corner::TopLeft);
            placed.push((label.text.clone(), Placement::TopLeft, corner));
        }
    }
    for (meter, (position, size)) in world.query::<(Entity, (&Position, &MeterSize))>().iter() {
        if measured(world, meter).is_empty() {
            let center = Frame::new(position.position, size).point(0.5, 0.5);
            placed.push((EMPTY_METER_HINT.to_string(), Placement::Center, center));
        }
    }
    for (speaker, position) in world
        .query::<(Entity, &Position)>()
        .with::<&Loudspeaker>()
        .iter()
    {
        if measured(world, speaker).is_empty() {
            placed.push((
                EMPTY_METER_HINT.to_string(),
                Placement::Beside(Side::Above),
                position.position,
            ));
        }
    }
    placed
}

fn lay_out(
    atlas: &mut GlyphAtlas,
    text: &str,
    placement: Placement,
) -> TessellatedGeometry<GlyphVertex> {
    let height = match placement {
        Placement::Beside(_) => LABEL_HEIGHT,
        Placement::TopLeft | Placement::Center => TITLE_HEIGHT,
    };
    match atlas.lay_out_wrapped(text, height, SINGLE_LINE) {
        Ok(wrapped) => match placement {
            Placement::Beside(side) => {
                let size = Vec2::new(wrapped.width, wrapped.height);
                let area = side.area(Vec2::ZERO, size);
                shifted(&wrapped.geometry, area.low.x, area.high.y)
            }
            Placement::TopLeft => shifted(&wrapped.geometry, TITLE_INSET, -TITLE_INSET * 0.5),
            Placement::Center => shifted(
                &wrapped.geometry,
                -wrapped.width * 0.5,
                wrapped.height * 0.5,
            ),
        },
        Err(e) => {
            log::error!("Failed to lay out a label: {e:?}");
            TessellatedGeometry::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_synapse, spawn_voltmeter, spawn_voltmeter_wire, Element, Meter};
    use crate::rendering::colors::Theme;
    use crate::rendering::{collect_meter_wires, compartment_radii, connection_paths};
    use hecs::World;

    fn label(world: &mut World, entity: hecs::Entity, text: &str) {
        world.get::<&mut Label>(entity).unwrap().text = text.to_string();
    }

    fn labelled(world: &mut World, at: Vec3, text: &str) -> Entity {
        let neuron = Element::ExcitatoryNeuron.spawn(world, at);
        label(world, neuron, text);
        neuron
    }

    fn placed(world: &World) -> Vec<(String, Placement, Vec3)> {
        let palette = Theme::default().palette();
        let radii = compartment_radii(world);
        let paths = connection_paths(world, &radii, &palette);
        let wires = collect_meter_wires(world, &palette);
        let drawn = Drawn {
            radii: &radii,
            paths: &paths,
            wires: &wires,
        };
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        placed_labels(world, &drawn, |text| measure_label(&mut atlas, text))
    }

    fn side_of(world: &World, neuron: Entity) -> Side {
        let at = world.get::<&Position>(neuron).unwrap().position;
        placed(world)
            .into_iter()
            .find_map(|(_, placement, anchor)| match placement {
                Placement::Beside(side) if anchor == at => Some(side),
                _ => None,
            })
            .unwrap()
    }

    fn input_at(world: &mut World, at: Vec3, target: Entity) {
        let input = Element::ExcitatoryNeuron.spawn(world, at);
        spawn_synapse(world, input, target);
    }

    #[test]
    fn labels_sit_centred_above_their_neuron() {
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        let laid = lay_out(&mut atlas, "Output", Placement::Beside(Side::Above));
        let xs = laid.vertices.iter().map(|v| v.position[0]);
        let ys = laid.vertices.iter().map(|v| v.position[1]);
        let (left, right) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let lowest = ys.fold(f32::MAX, f32::min);
        assert!((left + right).abs() < 0.1, "{left} {right}");
        assert!(lowest > NODE_RADIUS, "{lowest}");
    }

    #[test]
    fn labels_on_each_side_are_laid_out_where_they_were_placed() {
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        let measure = measure_label(&mut atlas, "Output");
        for side in SIDES {
            let laid = lay_out(&mut atlas, "Output", Placement::Beside(side));
            let ink = measure.ink_on(side, Vec2::ZERO);
            let corners = laid
                .vertices
                .iter()
                .map(|vertex| Vec2::from(vertex.position));
            let low = corners.clone().fold(Vec2::MAX, Vec2::min);
            let high = corners.fold(Vec2::MIN, Vec2::max);
            assert!(low.abs_diff_eq(ink.low, 1e-4), "{side:?} {low} {ink:?}");
            assert!(high.abs_diff_eq(ink.high, 1e-4), "{side:?} {high} {ink:?}");
            assert!(side
                .area(Vec2::ZERO, measure.line)
                .grown(0.1)
                .overlaps(&ink));
        }
    }

    #[test]
    fn a_label_stays_above_its_neuron_when_nothing_is_in_the_way() {
        let mut world = World::new();
        let alone = labelled(&mut world, Vec3::ZERO, "Alone");
        assert_eq!(side_of(&world, alone), Side::Above);
        let target = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::Z * 8.0);
        spawn_synapse(&mut world, alone, target);
        assert_eq!(side_of(&world, alone), Side::Above);
    }

    #[test]
    fn a_label_moves_below_synapses_that_arrive_from_above() {
        let mut world = World::new();
        let output = labelled(&mut world, Vec3::ZERO, "Output");
        for z in [-4.0, 0.0, 4.0] {
            input_at(&mut world, Vec3::new(6.0, 0.0, z), output);
        }
        assert_eq!(side_of(&world, output), Side::Below);
    }

    #[test]
    fn a_label_in_a_chain_from_top_to_bottom_moves_right_and_then_left() {
        let mut world = World::new();
        let middle = labelled(&mut world, Vec3::ZERO, "Middle");
        input_at(&mut world, Vec3::X * 6.0, middle);
        let below = Element::ExcitatoryNeuron.spawn(&mut world, -Vec3::X * 6.0);
        spawn_synapse(&mut world, middle, below);
        assert_eq!(side_of(&world, middle), Side::Right);
        input_at(&mut world, Vec3::Z * 6.0, middle);
        assert_eq!(side_of(&world, middle), Side::Left);
    }

    #[test]
    fn a_label_takes_the_least_crowded_side_when_every_side_is_taken() {
        let mut world = World::new();
        let crowded = labelled(&mut world, Vec3::ZERO, "Crowded");
        for at in [
            Vec3::new(6.0, 0.0, -1.0),
            Vec3::new(6.0, 0.0, 1.0),
            Vec3::X * -6.0,
            Vec3::Z * 8.0,
            Vec3::Z * -8.0,
        ] {
            input_at(&mut world, at, crowded);
        }
        assert_eq!(side_of(&world, crowded), Side::Below);
    }

    #[test]
    fn neighbouring_labels_keep_clear_of_each_other() {
        let mut world = World::new();
        let first = labelled(&mut world, Vec3::ZERO, "Neighbour");
        let second = labelled(&mut world, Vec3::Z * 3.0, "Neighbour");
        assert_eq!(side_of(&world, first), Side::Above);
        assert_eq!(side_of(&world, second), Side::Below);
    }

    #[test]
    fn labels_in_a_row_leave_each_other_room() {
        let mut world = World::new();
        let row = [0.0, 5.0].map(|z| {
            let neuron = labelled(&mut world, Vec3::Z * z, "Right");
            input_at(&mut world, Vec3::new(6.0, 0.0, z), neuron);
            let below = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(-6.0, 0.0, z));
            spawn_synapse(&mut world, neuron, below);
            neuron
        });
        let passing = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(6.0, 0.0, 7.5));
        input_at(&mut world, Vec3::new(-6.0, 0.0, 7.5), passing);
        assert_eq!(row.map(|neuron| side_of(&world, neuron)), [Side::Left; 2]);
    }

    #[test]
    fn a_label_keeps_clear_of_a_note() {
        let mut world = World::new();
        let neuron = labelled(&mut world, Vec3::ZERO, "Neuron");
        Element::Note.spawn(&mut world, Vec3::X * 4.0);
        assert_eq!(side_of(&world, neuron), Side::Below);
    }

    #[test]
    fn a_label_stays_on_one_line() {
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        let wrapped = atlas
            .lay_out_wrapped("Feedforward inhibition", LABEL_HEIGHT, SINGLE_LINE)
            .unwrap();
        let one_line = atlas
            .lay_out_wrapped("F", LABEL_HEIGHT, SINGLE_LINE)
            .unwrap();
        assert_eq!(wrapped.height, one_line.height);
    }

    #[test]
    fn labelled_neurons_also_title_their_voltmeter_plots() {
        let mut world = World::new();
        let a = labelled(&mut world, Vec3::ZERO, "A");
        let b = labelled(&mut world, Vec3::Z * 5.0, "B");
        let unlabelled = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::Z * 10.0);
        let voltmeter = spawn_voltmeter(&mut world, a, Vec3::X * 10.0);
        spawn_voltmeter_wire(&mut world, b, voltmeter);
        spawn_voltmeter_wire(&mut world, unlabelled, voltmeter);
        let frame = Frame::of(&world, voltmeter).unwrap();

        let mut titles: Vec<_> = placed(&world)
            .into_iter()
            .filter(|(_, placement, _)| *placement == Placement::TopLeft)
            .collect();
        titles.sort_by(|x, y| x.0.cmp(&y.0));
        assert_eq!(
            titles,
            [
                (
                    "A".to_string(),
                    Placement::TopLeft,
                    frame.row(0, 3).corner(Corner::TopLeft)
                ),
                (
                    "B".to_string(),
                    Placement::TopLeft,
                    frame.row(1, 3).corner(Corner::TopLeft)
                ),
            ]
        );
        side_of(&world, a);
        side_of(&world, b);
    }

    #[test]
    fn an_empty_meter_asks_for_a_neuron_until_it_measures_one() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let voltmeter = Meter::Voltmeter.spawn(&mut world, Vec3::X * 10.0);
        let speaker = Meter::Loudspeaker.spawn(&mut world, Vec3::Z * 10.0);
        let center = Frame::of(&world, voltmeter).unwrap().point(0.5, 0.5);
        let hints = |world: &World| -> Vec<(Placement, Vec3)> {
            placed(world)
                .into_iter()
                .filter(|(text, _, _)| text == EMPTY_METER_HINT)
                .map(|(_, placement, at)| (placement, at))
                .collect()
        };
        assert_eq!(
            hints(&world),
            [
                (Placement::Center, center),
                (Placement::Beside(Side::Above), Vec3::Z * 10.0)
            ]
        );

        spawn_voltmeter_wire(&mut world, neuron, voltmeter);
        Meter::Loudspeaker.wire(&mut world, neuron, speaker);
        assert!(hints(&world).is_empty());
    }

    #[test]
    fn the_empty_meter_hint_is_centred() {
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        let laid = lay_out(&mut atlas, EMPTY_METER_HINT, Placement::Center);
        let xs = laid.vertices.iter().map(|v| v.position[0]);
        let ys = laid.vertices.iter().map(|v| v.position[1]);
        let (left, right) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let (low, high) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        assert!((left + right).abs() < 0.1, "{left} {right}");
        assert!((low + high).abs() < 0.2, "{low} {high}");
    }
}
