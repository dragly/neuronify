use std::collections::VecDeque;
use std::ops::RangeInclusive;

use egui::{
    pos2, style::HandleShape, vec2, Align, Align2, Color32, Context, CornerRadius, FontId, Frame,
    Id, Layout, Margin, Order, Rect, Response, RichText, ScrollArea, Sense, Shape, Slider,
    SliderClamping, Stroke, Style, Ui, Vec2, WidgetInfo, WidgetType,
};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::MAX_ENDING_STRENGTH;
use crate::dendrites::compartment_tree;
use crate::editor::Editor;
use crate::elements::{axon_ending, Element};
use crate::measurement::Meter;
use crate::note;
use crate::rendering::Theme;
use crate::settings::Setting;
use crate::simulation::model::has_dendrites;
use crate::stimuli::Stimulus;
use crate::tools::Tool;

use super::icons::{paint_icon, paint_tool, Icon};
use super::sheet::{self, Sheet, HANDLE_HEIGHT};
use super::style::{Chrome, MARGIN};
use super::text_field::{covered_by_keyboard, text_field, Lines};
use super::widgets::{heading, segmented, segmented_width, text_width, ROW_RADIUS};

pub const INSPECTOR_WIDTH: f32 = 296.0;
pub const DELETE: &str = "inspector_delete";
pub const REVERSE: &str = "inspector_reverse";
pub const CLOSE: &str = "inspector_close";
pub const LABEL: &str = "inspector_label";
const PADDING: i8 = 14;
const ROOM_BELOW: f32 = 80.0;
const SHEET_PADDING: i8 = 16;
const PEEKING_SHEET: f32 = 212.0;
const EXPANDED_SHEET: f32 = 0.72;
const HISTORY_LENGTH: usize = 240;
const SPARKLINE_HEIGHT: f32 = 40.0;
const SPIKE_HEIGHT: Millivolts = Millivolts(20.0);
const HEADER_GLYPH: f32 = 30.0;
const HEADER_BUTTON: f32 = 32.0;
const HEADER_ICON: f32 = 18.0;
const BUTTON_HEIGHT: f32 = 36.0;
const BUTTON_GAP: f32 = 8.0;
const DESCRIPTION_SIZE: f32 = 11.5;
const SWITCH_HEIGHT: f32 = 30.0;
const POTENTIALS: RangeInclusive<Millivolts> = Millivolts(-100.0)..=Millivolts(50.0);
const POTENTIAL_GAP: Millivolts = Millivolts(0.05);
const REFRACTORY_PERIODS: RangeInclusive<Milliseconds> = Milliseconds(0.0)..=Milliseconds(50.0);
const RESISTANCES: (RangeInclusive<Megaohms>, Megaohms) =
    (Megaohms(1.0)..=Megaohms(200.0), Megaohms(1.0));
const CAPACITANCES: (RangeInclusive<Nanofarads>, Nanofarads) =
    (Nanofarads(0.01)..=Nanofarads(2.0), Nanofarads(0.01));
const SOMA_DIAMETERS: (RangeInclusive<Micrometres>, Micrometres) =
    (Micrometres(5.0)..=Micrometres(100.0), Micrometres(1.0));
const ADAPTATIONS: (RangeInclusive<Microsiemens>, Microsiemens) =
    (Microsiemens(0.0)..=Microsiemens(0.1), Microsiemens(0.0001));
const ADAPTATION_TIME_CONSTANTS: RangeInclusive<Milliseconds> =
    Milliseconds(0.0)..=Milliseconds(600.0);
const SYNAPSE_CURRENTS: RangeInclusive<Nanoamperes> = Nanoamperes(0.0)..=Nanoamperes(10.0);
const SYNAPSE_TIME_CONSTANTS: RangeInclusive<Milliseconds> = Milliseconds(0.1)..=Milliseconds(6.0);
const DELAYS: RangeInclusive<Milliseconds> = Milliseconds(0.0)..=Milliseconds(30.0);
const STRENGTHS: RangeInclusive<f64> = 0.0..=MAX_ENDING_STRENGTH;

enum Action {
    Delete,
    Close,
}

pub enum Placement {
    Panel,
    Sheet,
}

#[derive(Default)]
pub struct Inspector {
    entity: Option<Entity>,
    history: VecDeque<Millivolts>,
    last_time: Milliseconds,
    last_time_since_fire: Milliseconds,
    sheet: Sheet,
}

impl Inspector {
    pub fn show(
        &mut self,
        context: &Context,
        theme: Theme,
        world: &mut World,
        editor: &mut Editor,
        time: Milliseconds,
        placement: Placement,
    ) {
        let selected = editor.selection.filter(|&entity| world.contains(entity));
        let Some((entity, (tool, name))) =
            selected.and_then(|entity| Some((entity, kind(world, entity)?)))
        else {
            *self = Inspector::default();
            return;
        };
        if self.entity != Some(entity) {
            *self = Inspector {
                entity: Some(entity),
                last_time: time,
                last_time_since_fire: Milliseconds::NEG_INFINITY,
                sheet: std::mem::take(&mut self.sheet),
                ..Default::default()
            };
        }
        if time != self.last_time {
            self.last_time = time;
            self.record(world, entity);
        }
        let chrome = Chrome::of(theme);
        let covered = covered_by_keyboard();
        let mut screen = context.content_rect();
        screen.max.y -= covered;
        let settled = !context.egui_wants_keyboard_input()
            && !context.input(|input| input.pointer.any_down());
        let mut action = None;
        let mut dismissed = false;
        let mut contents = |ui: &mut Ui, room: f32, inspector: &mut Inspector| {
            apply_style(ui.style_mut(), chrome);
            if header(ui, chrome, theme, tool, name).clicked() {
                action = Some(Action::Close);
            }
            ScrollArea::vertical()
                .max_height(room)
                .min_scrolled_height(room)
                .show(ui, |ui| {
                    inspector.live(ui, chrome, world, entity);
                    editor.adjust(world, entity, settled, || {
                        parameters(ui, chrome, world, entity)
                    });
                    if world.get::<&Deletable>(entity).is_ok()
                        && delete_button(ui, chrome).clicked()
                    {
                        action = Some(Action::Delete);
                    }
                });
        };
        match placement {
            Placement::Panel => {
                egui::Area::new(Id::new("inspector"))
                    .anchor(Align2::RIGHT_TOP, [-MARGIN, MARGIN])
                    .show(context, |ui| {
                        chrome.panel_frame(PADDING).show(ui, |ui| {
                            ui.set_width(INSPECTOR_WIDTH - 2.0 * PADDING as f32);
                            let room = screen.height()
                                - MARGIN
                                - ROOM_BELOW
                                - 2.0 * PADDING as f32
                                - HEADER_BUTTON;
                            contents(ui, room, self);
                        });
                    });
            }
            Placement::Sheet => {
                let id = Id::new("inspector_sheet");
                let detents = [
                    PEEKING_SHEET,
                    (screen.height() * EXPANDED_SHEET).max(PEEKING_SHEET),
                ];
                let height = self.sheet.height(context, id, &detents);
                let (laid_out, dropped) = sheet::layout(height, &detents);
                egui::Area::new(id)
                    .order(Order::Foreground)
                    .anchor(Align2::LEFT_BOTTOM, [0.0, dropped])
                    .show(context, |ui| {
                        chrome.sheet_frame(SHEET_PADDING).show(ui, |ui| {
                            ui.set_width(screen.width() - 2.0 * SHEET_PADDING as f32);
                            ui.set_min_height(laid_out + covered - SHEET_PADDING as f32);
                            dismissed = self.sheet.handle(ui, chrome, height, &detents);
                            let room =
                                laid_out - HANDLE_HEIGHT - HEADER_BUTTON - SHEET_PADDING as f32;
                            contents(ui, room, self);
                        });
                    });
            }
        }
        if dismissed {
            action = Some(Action::Close);
        }
        match action {
            Some(Action::Delete) => editor.erase_selection(world),
            Some(Action::Close) => editor.selection = None,
            None => {}
        }
    }

    fn record(&mut self, world: &World, entity: Entity) {
        let sample = if let (Ok(neuron), Ok(state)) = (
            world.get::<&LeakyNeuron>(entity),
            world.get::<&NodeState>(entity),
        ) {
            let fired = state.since_spike < self.last_time_since_fire;
            self.last_time_since_fire = state.since_spike;
            Some(if fired {
                neuron.threshold + SPIKE_HEIGHT
            } else {
                state.voltage
            })
        } else {
            world
                .get::<&NodeState>(entity)
                .ok()
                .filter(|_| world.satisfies::<&Compartment>(entity))
                .map(|state| state.voltage)
        };
        if let Some(sample) = sample {
            if self.history.len() == HISTORY_LENGTH {
                self.history.pop_front();
            }
            self.history.push_back(sample);
        }
    }

    fn live(&self, ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) {
        let readout = Meter::of(world, entity).and_then(|meter| meter.readout(world, entity));
        if let Some((label, value)) = readout {
            Frame::new()
                .fill(chrome.raised)
                .corner_radius(ROW_RADIUS + 2)
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    reading(ui, chrome, label, value, 17.0);
                });
            ui.add_space(12.0);
            return;
        }
        let compartment = world.satisfies::<&Compartment>(entity);
        let (voltage, threshold) = if let (Ok(neuron), Ok(state)) = (
            world.get::<&LeakyNeuron>(entity),
            world.get::<&NodeState>(entity),
        ) {
            (millivolts(state.voltage), Some(neuron.threshold))
        } else if let (true, Ok(state)) = (compartment, world.get::<&NodeState>(entity)) {
            (millivolts(state.voltage), None)
        } else {
            return;
        };
        Frame::new()
            .fill(chrome.raised)
            .corner_radius(ROW_RADIUS + 2)
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                reading(ui, chrome, "Voltage", voltage, 17.0);
                if !self.history.is_empty() {
                    sparkline(ui, chrome, &self.history, threshold);
                }
            });
        ui.add_space(12.0);
    }
}

fn kind(world: &World, entity: Entity) -> Option<(Tool, &'static str)> {
    let has = |present: bool, element: Element| present.then_some(Tool::Place(element));
    let tool = if world.get::<&LeakyNeuron>(entity).is_ok() {
        has(
            world.get::<&AdaptationCurrent>(entity).is_ok(),
            Element::AdaptiveNeuron,
        )
        .or_else(|| {
            has(
                world.get::<&BurstCurrent>(entity).is_ok(),
                Element::BurstNeuron,
            )
        })
        .or_else(|| {
            has(
                neuron_type(world, entity) == NeuronType::Inhibitory,
                Element::InhibitoryNeuron,
            )
        })
        .unwrap_or(Tool::Place(Element::ExcitatoryNeuron))
    } else if world.get::<&Dendrite>(entity).is_ok() {
        return Some((Tool::Dendrite, "Dendrite"));
    } else if world.get::<&Compartment>(entity).is_ok() {
        let name = match axon_ending(world, entity) {
            Some(_) => "Axon terminal",
            None => "Compartment",
        };
        return Some((Tool::Axon, name));
    } else if world.get::<&CurrentSynapse>(entity).is_ok() {
        return Some((Tool::StaticConnection, "Synapse"));
    } else if world.get::<&Connection>(entity).is_ok() {
        return Some((Tool::StaticConnection, "Connection"));
    } else if let Some(stimulus) = Stimulus::of(world, entity) {
        Tool::Place(Element::Stimulus(stimulus))
    } else if let Some(meter) = Meter::of(world, entity) {
        Tool::Meter(meter)
    } else {
        return Some((Tool::Place(Element::Note), note::gui::title(world, entity)?));
    };
    Some((tool, tool.name()))
}

fn apply_style(style: &mut Style, chrome: &Chrome) {
    style.spacing.slider_rail_height = 4.0;
    style.spacing.item_spacing = vec2(6.0, 2.0);
    let visuals = &mut style.visuals;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = HandleShape::Circle;
    visuals.selection.bg_fill = chrome.accent.gamma_multiply(0.6);
    visuals.selection.stroke = Stroke::new(1.0_f32, chrome.accent);
    visuals.extreme_bg_color = chrome.raised;
    visuals.override_text_color = Some(chrome.text);
    visuals.text_cursor.stroke.color = chrome.accent;
    visuals.widgets.inactive.bg_fill = chrome.hovered;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, chrome.border);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(2);
    visuals.widgets.hovered.bg_fill = chrome.panel;
    visuals.widgets.active.bg_fill = chrome.accent;
    for state in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        state.fg_stroke = Stroke::new(2.0_f32, chrome.accent);
    }
}

fn header(ui: &mut Ui, chrome: &Chrome, theme: Theme, tool: Tool, name: &str) -> Response {
    let (row, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), HEADER_BUTTON), Sense::hover());
    let glyph = Rect::from_min_size(
        pos2(row.left(), row.center().y - HEADER_GLYPH / 2.0),
        Vec2::splat(HEADER_GLYPH),
    );
    paint_tool(ui.painter(), tool, glyph, theme, chrome.text, false);
    ui.painter().text(
        pos2(glyph.right() + 10.0, row.center().y),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(15.0),
        chrome.text,
    );
    let close = Rect::from_min_size(
        pos2(row.right() - HEADER_BUTTON, row.top()),
        Vec2::splat(HEADER_BUTTON),
    );
    let response = ui.interact(close, Id::new(CLOSE), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(close, ROW_RADIUS, chrome.raised);
    }
    paint_icon(
        ui.painter(),
        Icon::Close,
        Rect::from_center_size(close.center(), Vec2::splat(HEADER_ICON)),
        if response.hovered() {
            chrome.text
        } else {
            chrome.subtext
        },
    );
    ui.add_space(10.0);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Close"));
    response.on_hover_text("Close")
}

fn delete_button(ui: &mut Ui, chrome: &Chrome) -> Response {
    ui.add_space(6.0);
    row_button(ui, chrome, DELETE, Icon::Trash, "Delete", chrome.danger)
}

pub(crate) fn row_button(
    ui: &mut Ui,
    chrome: &Chrome,
    id: &str,
    icon: Icon,
    text: &str,
    color: Color32,
) -> Response {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), BUTTON_HEIGHT), Sense::hover());
    let response = ui.interact(rect, Id::new(id), Sense::click());
    ui.painter().rect_filled(
        rect,
        ROW_RADIUS,
        if response.hovered() {
            chrome.hovered
        } else {
            chrome.raised
        },
    );
    let label = ui
        .painter()
        .layout_no_wrap(text.to_owned(), FontId::proportional(13.5), color);
    let left = rect.center().x - (HEADER_ICON + BUTTON_GAP + label.size().x) / 2.0;
    paint_icon(
        ui.painter(),
        icon,
        Rect::from_min_size(
            pos2(left, rect.center().y - HEADER_ICON / 2.0),
            Vec2::splat(HEADER_ICON),
        ),
        color,
    );
    ui.painter().galley(
        pos2(
            left + HEADER_ICON + BUTTON_GAP,
            rect.center().y - label.size().y / 2.0,
        ),
        label,
        color,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, text));
    response
}

fn sparkline(
    ui: &mut Ui,
    chrome: &Chrome,
    history: &VecDeque<Millivolts>,
    threshold: Option<Millivolts>,
) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), SPARKLINE_HEIGHT), Sense::hover());
    let (low, high) = history.iter().chain(&threshold).fold(
        (Millivolts::INFINITY, Millivolts::NEG_INFINITY),
        |(low, high), &value| (low.min(value), high.max(value)),
    );
    let padding = ((high - low) * 0.08).max(Millivolts(1e-6));
    let (low, high) = (low - padding, high + padding);
    let y =
        |value: Millivolts| rect.bottom() - ((value - low) / (high - low)) as f32 * rect.height();
    let step = rect.width() / (HISTORY_LENGTH - 1) as f32;
    let first = HISTORY_LENGTH - history.len();
    if let Some(threshold) = threshold {
        ui.painter().extend(Shape::dashed_line(
            &[
                pos2(rect.left(), y(threshold)),
                pos2(rect.right(), y(threshold)),
            ],
            Stroke::new(1.0_f32, chrome.subtext.gamma_multiply(0.7)),
            3.0,
            3.0,
        ));
    }
    ui.painter().add(Shape::line(
        history
            .iter()
            .enumerate()
            .map(|(index, &value)| pos2(rect.left() + (first + index) as f32 * step, y(value)))
            .collect(),
        Stroke::new(1.6_f32, chrome.accent),
    ));
}

pub(crate) fn milliseconds(duration: Milliseconds) -> String {
    format!("{:.1} ms", duration.0)
}

pub(crate) fn nanoamperes(current: Nanoamperes) -> String {
    format!("{:.2} nA", current.0)
}

pub(crate) fn picoamperes(current: Nanoamperes) -> String {
    format!("{:.0} pA", current.in_picoamperes())
}

pub(crate) fn millivolts(voltage: Millivolts) -> String {
    format!("{:.1} mV", voltage.0).replace('-', "\u{2212}")
}

pub(crate) fn hertz(rate: Hertz) -> String {
    format!("{:.0} Hz", rate.0)
}

fn values<Q: Quantity>(range: RangeInclusive<Q>) -> RangeInclusive<f64> {
    range.start().value()..=range.end().value()
}

pub(crate) fn parameter<Q: Quantity>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut Q,
    range: RangeInclusive<Q>,
    shown: impl Fn(Q) -> String,
) -> bool {
    slider_row(
        ui,
        chrome,
        setting,
        value,
        |value| Slider::new(value, values(range)),
        shown,
    )
}

pub(crate) fn logarithmic_parameter<Q: Quantity>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut Q,
    (range, smallest_positive): (RangeInclusive<Q>, Q),
    shown: impl Fn(Q) -> String,
) -> bool {
    slider_row(
        ui,
        chrome,
        setting,
        value,
        |value| {
            Slider::new(value, values(range))
                .logarithmic(true)
                .smallest_positive(smallest_positive.value())
        },
        shown,
    )
}

fn slider_row<Q: Quantity>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut Q,
    slider: impl for<'a> FnOnce(&'a mut f64) -> Slider<'a>,
    shown: impl Fn(Q) -> String,
) -> bool {
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    ui.spacing_mut().slider_width = ui.available_width();
    let changed = ui
        .add(
            slider(value.value_mut())
                .show_value(false)
                .clamping(SliderClamping::Edits),
        )
        .changed();
    ui.painter().text(
        row.left_center(),
        Align2::LEFT_CENTER,
        setting.label,
        FontId::proportional(12.5),
        chrome.subtext,
    );
    ui.painter().text(
        row.right_center(),
        Align2::RIGHT_CENTER,
        shown(*value),
        FontId::monospace(12.5),
        chrome.text,
    );
    description(ui, chrome, setting.description);
    ui.add_space(8.0);
    changed
}

pub(crate) fn description(ui: &mut Ui, chrome: &Chrome, text: &str) {
    let galley = ui.painter().layout(
        text.to_owned(),
        FontId::proportional(DESCRIPTION_SIZE),
        chrome.subtext,
        ui.available_width(),
    );
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), galley.size().y + 2.0),
        Sense::hover(),
    );
    ui.painter()
        .galley(rect.left_top() + vec2(0.0, 2.0), galley, chrome.subtext);
}

fn reading(ui: &mut Ui, chrome: &Chrome, label: &str, value: String, size: f32) {
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), size + 6.0), Sense::hover());
    ui.painter().text(
        row.left_center(),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        chrome.subtext,
    );
    ui.painter().text(
        row.right_center(),
        Align2::RIGHT_CENTER,
        value,
        FontId::monospace(size),
        chrome.text,
    );
}

fn label_field(ui: &mut Ui, chrome: &Chrome, text: &mut String) -> bool {
    ui.label(
        RichText::new(Setting::LABEL.label)
            .size(12.5)
            .color(chrome.subtext),
    );
    let changed = text_field(ui, Id::new(LABEL), text, Lines::One, |edit| {
        edit.desired_width(f32::INFINITY)
            .margin(Margin::symmetric(8, 6))
    });
    description(ui, chrome, Setting::LABEL.description);
    ui.add_space(8.0);
    changed
}

pub(crate) fn choice<T: PartialEq + Copy>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    let size = 12.5;
    let label = RichText::new(setting.label)
        .size(size)
        .color(chrome.subtext);
    let beside = text_width(ui, setting.label, size)
        + ui.spacing().item_spacing.x
        + segmented_width(ui, options)
        <= ui.available_width();
    if beside {
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), SWITCH_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.label(label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    changed = segmented(ui, chrome, setting, value, options);
                });
            },
        );
    } else {
        ui.label(label);
        changed = segmented(ui, chrome, setting, value, options);
    }
    description(ui, chrome, setting.description);
    ui.add_space(8.0);
    changed
}

pub(crate) fn type_switch(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Some(current) = world.get::<&NeuronType>(entity).ok().map(|t| *t) else {
        return false;
    };
    let mut chosen = current;
    choice(
        ui,
        chrome,
        Setting::TYPE,
        &mut chosen,
        &[
            (NeuronType::Inhibitory, "Inhibitory"),
            (NeuronType::Excitatory, "Excitatory"),
        ],
    );
    if chosen == current {
        return false;
    }
    for part in std::iter::once(entity).chain(compartment_tree(world, entity)) {
        if let Ok(mut neuron_type) = world.get::<&mut NeuronType>(part) {
            *neuron_type = chosen;
        }
    }
    true
}

fn neuron_parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let mut changed = false;
    heading(ui, chrome, "General", 0.0);
    if let Ok(mut label) = world.get::<&mut Label>(entity) {
        changed |= label_field(ui, chrome, &mut label.text);
    }
    changed |= type_switch(ui, chrome, world, entity);
    if let Ok(mut soma) = world.get::<&mut CompartmentSoma>(entity) {
        heading(ui, chrome, "Potentials", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::SPIKE_THRESHOLD,
            &mut soma.threshold,
            POTENTIALS,
            millivolts,
        );
        if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
            changed |= parameter(
                ui,
                chrome,
                Setting::RESTING_POTENTIAL,
                &mut neuron.resting_potential,
                POTENTIALS,
                millivolts,
            );
            heading(ui, chrome, "Membrane", 0.0);
            changed |= soma_diameter(ui, chrome, &mut neuron);
        }
    } else {
        changed |= rule_parameters(ui, chrome, world, entity);
    }
    if let Ok(mut adaptation) = world.get::<&mut AdaptationCurrent>(entity) {
        heading(ui, chrome, "Adaptation", 0.0);
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::ADAPTATION,
            &mut adaptation.adaptation,
            ADAPTATIONS,
            |conductance| format!("{:.1} nS", conductance.in_nanosiemens()),
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::ADAPTATION_TIME_CONSTANT,
            &mut adaptation.time_constant,
            ADAPTATION_TIME_CONSTANTS,
            milliseconds,
        );
    }
    changed
}

fn rule_parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let mut changed = false;
    if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
        changed |= parameter(
            ui,
            chrome,
            Setting::REFRACTORY_PERIOD,
            &mut neuron.refractory_period,
            REFRACTORY_PERIODS,
            milliseconds,
        );
    }
    if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
        heading(ui, chrome, "Potentials", 0.0);
        let threshold = parameter(
            ui,
            chrome,
            Setting::THRESHOLD,
            &mut neuron.threshold,
            POTENTIALS,
            millivolts,
        );
        if threshold {
            neuron.threshold = neuron
                .threshold
                .max(neuron.initial_potential + POTENTIAL_GAP);
        }
        let reset = parameter(
            ui,
            chrome,
            Setting::RESET,
            &mut neuron.initial_potential,
            POTENTIALS,
            millivolts,
        );
        if reset {
            neuron.initial_potential = neuron
                .initial_potential
                .min(neuron.threshold - POTENTIAL_GAP);
        }
        changed |= threshold || reset;
        changed |= parameter(
            ui,
            chrome,
            Setting::RESTING_POTENTIAL,
            &mut neuron.resting_potential,
            POTENTIALS,
            millivolts,
        );
    }
    if has_dendrites(world, entity) {
        if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
            heading(ui, chrome, "Membrane", 0.0);
            changed |= soma_diameter(ui, chrome, &mut neuron);
        }
    } else if let (Ok(mut neuron), Ok(mut leak)) = (
        world.get::<&mut LeakyNeuron>(entity),
        world.get::<&mut LeakCurrent>(entity),
    ) {
        heading(ui, chrome, "Membrane", 0.0);
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::RESISTANCE,
            &mut leak.resistance,
            RESISTANCES,
            |resistance| format!("{:.0} M\u{3a9}", resistance.0),
        );
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::CAPACITANCE,
            &mut neuron.capacitance,
            CAPACITANCES,
            |capacitance| format!("{:.0} pF", capacitance.in_picofarads()),
        );
        reading(
            ui,
            chrome,
            Setting::TIME_CONSTANT.label,
            milliseconds(leak.resistance * neuron.capacitance),
            12.5,
        );
        description(ui, chrome, Setting::TIME_CONSTANT.description);
        ui.add_space(8.0);
    }
    changed
}

fn soma_diameter(ui: &mut Ui, chrome: &Chrome, neuron: &mut LeakyNeuron) -> bool {
    logarithmic_parameter(
        ui,
        chrome,
        Setting::SOMA_DIAMETER,
        &mut neuron.soma_diameter,
        SOMA_DIAMETERS,
        |diameter| format!("{:.0} \u{b5}m", diameter.0),
    )
}

fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let mut changed = false;
    if world.get::<&LeakyNeuron>(entity).is_ok() {
        changed |= neuron_parameters(ui, chrome, world, entity);
    }
    if let Some(meter) = Meter::of(world, entity) {
        changed |= meter.parameters(ui, chrome, world, entity);
    }
    if let Some(stimulus) = Stimulus::of(world, entity) {
        changed |= stimulus.parameters(ui, chrome, world, entity);
    }
    if let Ok(mut synapse) = world.get::<&mut CurrentSynapse>(entity) {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::MAXIMUM_CURRENT,
            &mut synapse.maximum_current,
            SYNAPSE_CURRENTS,
            nanoamperes,
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::SYNAPSE_TIME_CONSTANT,
            &mut synapse.tau,
            SYNAPSE_TIME_CONSTANTS,
            milliseconds,
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::DELAY,
            &mut synapse.delay,
            DELAYS,
            milliseconds,
        );
    }
    changed |= note::gui::parameters(ui, chrome, world, entity);
    let ending =
        axon_ending(world, entity).and_then(|ending| world.get::<&mut Connection>(ending).ok());
    if let Some(mut ending) = ending {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::STRENGTH,
            &mut ending.strength,
            STRENGTHS,
            |strength| format!("{strength:.2}"),
        );
    }
    let gates = world
        .get::<&NodeState>(entity)
        .ok()
        .filter(|state| state.gates.len() == 3)
        .map(|state| [state.gates[0], state.gates[2], state.gates[1]]);
    if let Some(gates) = gates {
        heading(ui, chrome, "Gating", 0.0);
        for (name, gate) in ["m", "n", "h"].into_iter().zip(gates) {
            reading(ui, chrome, name, format!("{gate:.2}"), 12.5);
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::bounds;

    fn inside<Q: Quantity>(slider: RangeInclusive<Q>, bound: RangeInclusive<Q>) -> bool {
        bound.contains(slider.start()) && bound.contains(slider.end())
    }

    #[test]
    fn every_slider_stays_within_what_a_save_file_may_hold() {
        assert!(inside(REFRACTORY_PERIODS, bounds::REFRACTORY_PERIODS));
        assert!(inside(POTENTIALS, bounds::POTENTIALS));
        assert!(inside(RESISTANCES.0, bounds::RESISTANCES));
        assert!(inside(CAPACITANCES.0, bounds::CAPACITANCES));
        assert!(inside(SOMA_DIAMETERS.0, bounds::SOMA_DIAMETERS));
        assert!(inside(ADAPTATIONS.0, bounds::ADAPTATIONS));
        assert!(inside(
            ADAPTATION_TIME_CONSTANTS,
            bounds::ADAPTATION_TIME_CONSTANTS
        ));
        assert!(inside(SYNAPSE_CURRENTS, bounds::SYNAPSE_CURRENTS));
        assert!(inside(
            SYNAPSE_TIME_CONSTANTS,
            bounds::SYNAPSE_TIME_CONSTANTS
        ));
        assert!(inside(DELAYS, bounds::DELAYS));
        assert!(inside(STRENGTHS, bounds::STRENGTHS));
    }
}
