use std::collections::HashMap;

use glam::{Vec3, Vec4};
use visula::{
    text::{GlyphAtlas, TessellatedGeometry},
    Expression, GlyphDelegate, GlyphVertex, Glyphs,
};

use crate::components::{Label, MeterSize, Position};
use crate::constants::NODE_RADIUS;
use crate::measurement::firing_rate::RatePlot;
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::colors::{srgb, Theme};
use crate::rendering::notes::{note_plane, shifted, translated, NOTE_FONT_DATA, NOTE_TEXT_Y};

pub const LABEL_HEIGHT: f32 = 0.8;
pub const LABEL_GAP: f32 = 0.3;
const TITLE_HEIGHT: f32 = 0.55;
const TITLE_INSET: f32 = 0.25;
const SINGLE_LINE: f32 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Placement {
    Above,
    TopLeft,
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
    uploaded: Vec<(String, Placement, Vec3)>,
}

impl LabelRenderer {
    pub fn new(application: &visula::Application, theme: Theme) -> Self {
        Self {
            theme,
            text: glyphs(application, theme),
            atlas: GlyphAtlas::new(NOTE_FONT_DATA).unwrap(),
            laid: HashMap::new(),
            uploaded: Vec::new(),
        }
    }

    pub fn update(&mut self, application: &visula::Application, world: &hecs::World, theme: Theme) {
        if theme != self.theme {
            self.theme = theme;
            self.text = glyphs(application, theme);
            self.uploaded.clear();
        }
        let placed = placed_labels(world);
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

fn glyphs(application: &visula::Application, theme: Theme) -> Glyphs {
    Glyphs::new(
        &application.rendering_descriptor(),
        &GlyphDelegate {
            color: text_color(theme).into(),
            transform: Expression::from(note_plane(NOTE_TEXT_Y)),
        },
    )
    .unwrap()
}

fn placed_labels(world: &hecs::World) -> Vec<(String, Placement, Vec3)> {
    let mut placed: Vec<(String, Placement, Vec3)> = world
        .query::<(&Label, &Position)>()
        .iter()
        .filter(|(_, (label, _))| !label.text.is_empty())
        .map(|(_, (label, position))| (label.text.clone(), Placement::Above, position.position))
        .collect();
    for (_, (plot, position, size)) in world.query::<(&RatePlot, &Position, &MeterSize)>().iter() {
        let corner = Frame::new(position.position, size).corner(Corner::TopLeft);
        placed.push((
            format!("{:.0} Hz", plot.maximum),
            Placement::TopLeft,
            corner,
        ));
    }
    for (meter, _) in world
        .query::<()>()
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
    placed
}

fn lay_out(
    atlas: &mut GlyphAtlas,
    text: &str,
    placement: Placement,
) -> TessellatedGeometry<GlyphVertex> {
    let height = match placement {
        Placement::Above => LABEL_HEIGHT,
        Placement::TopLeft => TITLE_HEIGHT,
    };
    match atlas.lay_out_wrapped(text, height, SINGLE_LINE) {
        Ok(wrapped) => match placement {
            Placement::Above => shifted(
                &wrapped.geometry,
                -wrapped.width * 0.5,
                NODE_RADIUS + LABEL_GAP + wrapped.height,
            ),
            Placement::TopLeft => shifted(&wrapped.geometry, TITLE_INSET, -TITLE_INSET * 0.5),
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
    use crate::elements::{spawn_voltmeter, spawn_voltmeter_wire, Element};
    use hecs::World;

    fn label(world: &mut World, entity: hecs::Entity, text: &str) {
        world.get::<&mut Label>(entity).unwrap().text = text.to_string();
    }

    #[test]
    fn labels_sit_centred_above_their_neuron() {
        let mut atlas = GlyphAtlas::new(NOTE_FONT_DATA).unwrap();
        let laid = lay_out(&mut atlas, "Output", Placement::Above);
        let xs = laid.vertices.iter().map(|v| v.position[0]);
        let ys = laid.vertices.iter().map(|v| v.position[1]);
        let (left, right) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let lowest = ys.fold(f32::MAX, f32::min);
        assert!((left + right).abs() < 0.1, "{left} {right}");
        assert!(lowest > NODE_RADIUS, "{lowest}");
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
        let a = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let b = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::Z * 5.0);
        let unlabelled = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::Z * 10.0);
        label(&mut world, a, "A");
        label(&mut world, b, "B");
        let voltmeter = spawn_voltmeter(&mut world, a, Vec3::X * 10.0);
        spawn_voltmeter_wire(&mut world, b, voltmeter);
        spawn_voltmeter_wire(&mut world, unlabelled, voltmeter);
        let frame = Frame::of(&world, voltmeter).unwrap();

        let mut placed = placed_labels(&world);
        placed.sort_by(|x, y| x.0.cmp(&y.0).then(x.1.cmp(&y.1)));
        assert_eq!(
            placed,
            [
                ("A".to_string(), Placement::Above, Vec3::ZERO),
                (
                    "A".to_string(),
                    Placement::TopLeft,
                    frame.row(0, 3).corner(Corner::TopLeft)
                ),
                ("B".to_string(), Placement::Above, Vec3::Z * 5.0),
                (
                    "B".to_string(),
                    Placement::TopLeft,
                    frame.row(1, 3).corner(Corner::TopLeft)
                ),
            ]
        );
    }
}
