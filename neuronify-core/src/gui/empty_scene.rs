use egui::{vec2, Align2, Context, Id, Order, Pos2, Rect, RichText, Shape, Stroke, Vec2};
use hecs::World;
use rand::Rng;

use super::style::{Chrome, MARGIN};
use crate::components::Position;
use crate::rendering::Theme;
use crate::tools::Tool;

const ADD_CALLOUT: &str = "Press Add for neurons and more";
const NEURON_CALLOUT: &str = "Pick a neuron to start";
const HEADING: &str = "Nothing here yet";
pub const SUGGESTIONS: [&str; 12] = [
    "How about connecting a touch sensor to a neuron and watching it fire?",
    "Connect a current source to a neuron and watch it fire.",
    "Connect a visual input to a neuron. Which pictures make it fire?",
    "Ever heard a neuron fire? Put a loudspeaker on one and listen.",
    "What does a neuron's voltage do just before it fires? Put a voltmeter on it.",
    "Connect Regular spikes to a neuron. How fast can it keep up?",
    "Connect Random spikes to a neuron. Can you guess when it fires?",
    "Does an adaptive neuron get tired? Feed it a current source and find out.",
    "What does a bursting neuron do when you press a touch sensor?",
    "Can you send a spike down a chain of neurons?",
    "Can an inhibitory neuron stop another neuron from firing?",
    "Connect an alternating current to a neuron. When does it fire?",
];
const HEADING_SIZE: f32 = 22.0;
const INVITATION_SIZE: f32 = 15.0;
const LINE_GAP: f32 = 6.0;
const MESSAGE_WIDTH: f32 = 360.0;
const CALLOUT_PADDING: i8 = 8;
const CALLOUT_GAP: f32 = 10.0;
const POINTER: f32 = 7.0;

#[derive(Default)]
pub struct EmptyScene {
    suggestion: Option<&'static str>,
}

impl EmptyScene {
    pub fn forget(&mut self) {
        self.suggestion = None;
    }

    pub fn invitation(&mut self, world: &World, tool: Tool) -> Option<&'static str> {
        if world.query::<&Position>().iter().next().is_some() {
            self.suggestion = None;
            return None;
        }
        let suggestion = *self
            .suggestion
            .get_or_insert_with(|| SUGGESTIONS[rand::thread_rng().gen_range(0..SUGGESTIONS.len())]);
        let adding = !matches!(
            tool,
            Tool::View | Tool::Edit | Tool::Stimulate | Tool::Erase
        );
        (!adding).then_some(suggestion)
    }
}

pub fn message(context: &Context, theme: Theme, canvas: Rect, invitation: &str) {
    let chrome = Chrome::of(theme);
    let width = (canvas.width() - 2.0 * MARGIN).min(MESSAGE_WIDTH);
    egui::Area::new(Id::new("empty_scene"))
        .order(Order::Background)
        .interactable(false)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(canvas.center())
        .show(context, |ui| {
            ui.set_width(width);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(HEADING).size(HEADING_SIZE).color(chrome.text));
                ui.add_space(LINE_GAP);
                ui.label(
                    RichText::new(invitation)
                        .size(INVITATION_SIZE)
                        .color(chrome.subtext),
                );
            });
        });
}

pub fn callout(context: &Context, theme: Theme, add: Rect, above: bool) {
    let chrome = Chrome::of(theme);
    let (tip, toward, pivot, text) = if above {
        let tip = add.center_top() - vec2(0.0, CALLOUT_GAP);
        (tip, vec2(0.0, 1.0), Align2::CENTER_BOTTOM, ADD_CALLOUT)
    } else {
        let tip = add.right_center() + vec2(CALLOUT_GAP, 0.0);
        (tip, vec2(-1.0, 0.0), Align2::LEFT_CENTER, NEURON_CALLOUT)
    };
    let base = tip - toward * POINTER;
    let bubble = egui::Area::new(Id::new("add_callout"))
        .order(Order::Foreground)
        .interactable(false)
        .pivot(pivot)
        .fixed_pos(base)
        .constrain_to(context.content_rect().shrink(MARGIN))
        .show(context, |ui| {
            chrome.panel_frame(CALLOUT_PADDING).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(text).color(chrome.text)).extend());
            });
        });
    let side = vec2(toward.y, -toward.x) * POINTER;
    let corners = [base + side, tip, base - side];
    pointer(context, chrome, bubble.response.layer_id, corners, toward);
}

fn pointer(
    context: &Context,
    chrome: &Chrome,
    layer: egui::LayerId,
    [left, tip, right]: [Pos2; 3],
    toward: Vec2,
) {
    let painter = context.layer_painter(layer);
    let overlap = -toward * 1.5;
    painter.add(Shape::convex_polygon(
        vec![left + overlap, tip, right + overlap],
        chrome.panel,
        Stroke::NONE,
    ));
    let edge = Stroke::new(1.0_f32, chrome.border);
    painter.line_segment([left, tip], edge);
    painter.line_segment([tip, right], edge);
}
