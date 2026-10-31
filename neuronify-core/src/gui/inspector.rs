use std::collections::VecDeque;
use std::ops::RangeInclusive;

use egui::emath::Numeric;
use egui::{
    pos2, style::HandleShape, vec2, Align, Align2, Context, CornerRadius, FontId, Frame, Id,
    Layout, Margin, Order, Rect, Response, RichText, ScrollArea, Sense, Shape, Slider,
    SliderClamping, Stroke, Style, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::MAX_ENDING_STRENGTH;
use crate::dendrites::compartment_tree;
use crate::editor::Editor;
use crate::elements::{axon_ending, Element};
use crate::measurement::voltmeter::{measured, VoltageSeries, Voltmeter, VoltmeterRange};
use crate::rendering::Theme;
use crate::settings::Setting;
use crate::tools::Tool;

use super::icons::{paint_icon, paint_tool, Icon};
use super::sheet::{self, Sheet, HANDLE_HEIGHT};
use super::style::{Chrome, MARGIN};
use super::widgets::{heading, segmented, ROW_RADIUS};

pub const INSPECTOR_WIDTH: f32 = 296.0;
pub const DELETE: &str = "inspector_delete";
pub const CLOSE: &str = "inspector_close";
pub const LABEL: &str = "inspector_label";
const PADDING: i8 = 14;
const ROOM_BELOW: f32 = 80.0;
const SHEET_PADDING: i8 = 16;
const PEEKING_SHEET: f32 = 212.0;
const EXPANDED_SHEET: f32 = 0.72;
const HISTORY_LENGTH: usize = 240;
const SPARKLINE_HEIGHT: f32 = 40.0;
const SPIKE_HEIGHT: f64 = 0.02;
const HEADER_GLYPH: f32 = 30.0;
const HEADER_BUTTON: f32 = 32.0;
const HEADER_ICON: f32 = 18.0;
const DELETE_HEIGHT: f32 = 36.0;
const DELETE_GAP: f32 = 8.0;
const DESCRIPTION_SIZE: f32 = 11.5;
const SWITCH_HEIGHT: f32 = 30.0;
const POTENTIALS: RangeInclusive<f64> = -0.1..=0.05;
const POTENTIAL_GAP: f64 = 0.05e-3;
const REFRACTORY_PERIODS: RangeInclusive<f64> = 0.0..=0.05;
const RESISTANCES: (RangeInclusive<f64>, f64) = (1e6..=200e6, 1e6);
const CAPACITANCES: (RangeInclusive<f64>, f64) = (10e-12..=2e-9, 10e-12);
const ADAPTATIONS: (RangeInclusive<f64>, f64) = (0.0..=100e-9, 0.1e-9);
const ADAPTATION_TIME_CONSTANTS: RangeInclusive<f64> = 0.0..=0.6;
const PLOTTED_VOLTAGES: RangeInclusive<f64> = -250.0..=250.0;
const PLOTTED_VOLTAGE_GAP: f64 = 1.0;
const SYNAPSE_CURRENTS: RangeInclusive<f64> = 0.0..=10e-9;
const SYNAPSE_TIME_CONSTANTS: RangeInclusive<f64> = 0.1e-3..=6e-3;
const DELAYS: RangeInclusive<f64> = 0.0..=30e-3;
const CURRENTS: (RangeInclusive<f64>, f64) = (0.0..=30e-9, 0.01e-9);
const SPIKE_RATES: (RangeInclusive<f64>, f64) = (0.0..=1000.0, 1.0);

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
    history: VecDeque<f64>,
    last_time: f64,
    last_time_since_fire: f64,
    sheet: Sheet,
}

impl Inspector {
    pub fn show(
        &mut self,
        context: &Context,
        theme: Theme,
        world: &mut World,
        editor: &mut Editor,
        time: f64,
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
                last_time_since_fire: f64::NEG_INFINITY,
                sheet: std::mem::take(&mut self.sheet),
                ..Default::default()
            };
        }
        if time != self.last_time {
            self.last_time = time;
            self.record(world, entity);
        }
        let chrome = Chrome::of(theme);
        let screen = context.content_rect();
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
                    let adjusted = axon_ending(world, entity).unwrap_or(entity);
                    editor.adjust(world, adjusted, settled, || {
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
                            ui.set_min_height(laid_out - SHEET_PADDING as f32);
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
        let sample = if let (Ok(neuron), Ok(dynamics)) = (
            world.get::<&LeakyNeuron>(entity),
            world.get::<&LeakyDynamics>(entity),
        ) {
            let fired = dynamics.time_since_fire < self.last_time_since_fire;
            self.last_time_since_fire = dynamics.time_since_fire;
            Some(if fired {
                neuron.threshold + SPIKE_HEIGHT
            } else {
                dynamics.voltage
            })
        } else {
            world
                .get::<&Compartment>(entity)
                .ok()
                .map(|compartment| compartment.voltage)
        };
        if let Some(sample) = sample {
            if self.history.len() == HISTORY_LENGTH {
                self.history.pop_front();
            }
            self.history.push_back(sample);
        }
    }

    fn live(&self, ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) {
        let (voltage, threshold) = if let (Ok(neuron), Ok(dynamics)) = (
            world.get::<&LeakyNeuron>(entity),
            world.get::<&LeakyDynamics>(entity),
        ) {
            (millivolts(dynamics.voltage), Some(neuron.threshold))
        } else if let Ok(compartment) = world.get::<&Compartment>(entity) {
            (format!("{:.2}", compartment.voltage), None)
        } else if world.get::<&Voltmeter>(entity).is_ok() {
            let first = measured(world, entity).first().map(|&(wire, _)| wire);
            let Some(reading) = first.and_then(|wire| {
                let series = world.get::<&VoltageSeries>(wire).ok()?;
                Some(format!("{:.1} mV", series.measurements.last()?.voltage))
            }) else {
                return;
            };
            (reading.replace('-', "\u{2212}"), None)
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
    } else if world.get::<&CurrentClamp>(entity).is_ok() {
        Tool::Place(Element::CurrentSource)
    } else if world.get::<&TouchSensor>(entity).is_ok() {
        Tool::Place(Element::TouchSensor)
    } else if world.get::<&RegularSpikeGenerator>(entity).is_ok() {
        Tool::Place(Element::RegularSpikeGenerator)
    } else if world.get::<&PoissonGenerator>(entity).is_ok() {
        Tool::Place(Element::PoissonGenerator)
    } else if world.get::<&Voltmeter>(entity).is_ok() {
        Tool::Voltmeter
    } else if world.get::<&Link>(entity).is_ok() {
        return Some((Tool::Place(Element::Note), "Link"));
    } else if world.get::<&Annotation>(entity).is_ok() {
        Tool::Place(Element::Note)
    } else {
        return None;
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
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), DELETE_HEIGHT), Sense::hover());
    let response = ui.interact(rect, Id::new(DELETE), Sense::click());
    ui.painter().rect_filled(
        rect,
        ROW_RADIUS,
        if response.hovered() {
            chrome.hovered
        } else {
            chrome.raised
        },
    );
    let label = ui.painter().layout_no_wrap(
        "Delete".to_owned(),
        FontId::proportional(13.5),
        chrome.danger,
    );
    let left = rect.center().x - (HEADER_ICON + DELETE_GAP + label.size().x) / 2.0;
    paint_icon(
        ui.painter(),
        Icon::Trash,
        Rect::from_min_size(
            pos2(left, rect.center().y - HEADER_ICON / 2.0),
            Vec2::splat(HEADER_ICON),
        ),
        chrome.danger,
    );
    ui.painter().galley(
        pos2(
            left + HEADER_ICON + DELETE_GAP,
            rect.center().y - label.size().y / 2.0,
        ),
        label,
        chrome.danger,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Delete"));
    response
}

fn sparkline(ui: &mut Ui, chrome: &Chrome, history: &VecDeque<f64>, threshold: Option<f64>) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), SPARKLINE_HEIGHT), Sense::hover());
    let (low, high) = history
        .iter()
        .chain(&threshold)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), &value| {
            (low.min(value), high.max(value))
        });
    let padding = ((high - low) * 0.08).max(1e-6);
    let (low, high) = (low - padding, high + padding);
    let y = |value: f64| rect.bottom() - ((value - low) / (high - low)) as f32 * rect.height();
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

fn milliseconds(seconds: f64) -> String {
    format!("{:.1} ms", seconds * 1e3)
}

fn nanoamperes(amperes: f64) -> String {
    format!("{:.2} nA", amperes * 1e9)
}

fn millivolts(volts: f64) -> String {
    format!("{:.1} mV", volts * 1e3).replace('-', "\u{2212}")
}

fn parameter<N: Numeric>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut N,
    range: RangeInclusive<N>,
    shown: impl Fn(N) -> String,
) -> bool {
    slider_row(
        ui,
        chrome,
        setting,
        value,
        |value| Slider::new(value, range),
        shown,
    )
}

fn logarithmic_parameter(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut f64,
    (range, smallest_positive): (RangeInclusive<f64>, f64),
    shown: impl Fn(f64) -> String,
) -> bool {
    slider_row(
        ui,
        chrome,
        setting,
        value,
        |value| {
            Slider::new(value, range)
                .logarithmic(true)
                .smallest_positive(smallest_positive)
        },
        shown,
    )
}

fn slider_row<N: Numeric>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut N,
    slider: impl for<'a> FnOnce(&'a mut N) -> Slider<'a>,
    shown: impl Fn(N) -> String,
) -> bool {
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    ui.spacing_mut().slider_width = ui.available_width();
    let changed = ui
        .add(
            slider(value)
                .show_value(false)
                .clamping(SliderClamping::Edits),
        )
        .changed();
    ui.painter().text(
        row.left_center(),
        Align2::LEFT_CENTER,
        setting.label(),
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
    description(ui, chrome, setting.description());
    ui.add_space(8.0);
    changed
}

fn description(ui: &mut Ui, chrome: &Chrome, text: &str) {
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
        RichText::new(Setting::Label.label())
            .size(12.5)
            .color(chrome.subtext),
    );
    let changed = ui
        .add(
            TextEdit::singleline(text)
                .id(Id::new(LABEL))
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(8, 6)),
        )
        .changed();
    description(ui, chrome, Setting::Label.description());
    ui.add_space(8.0);
    changed
}

fn choice<T: PartialEq + Copy>(
    ui: &mut Ui,
    chrome: &Chrome,
    setting: Setting,
    value: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), SWITCH_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                RichText::new(setting.label())
                    .size(12.5)
                    .color(chrome.subtext),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                changed = segmented(ui, chrome, value, options);
            });
        },
    );
    description(ui, chrome, setting.description());
    ui.add_space(8.0);
    changed
}

fn type_switch(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let Some(current) = world.get::<&NeuronType>(entity).ok().map(|t| *t) else {
        return false;
    };
    let mut chosen = current;
    choice(
        ui,
        chrome,
        Setting::Type,
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
    if let Ok(mut dynamics) = world.get::<&mut LeakyDynamics>(entity) {
        changed |= parameter(
            ui,
            chrome,
            Setting::RefractoryPeriod,
            &mut dynamics.refractory_period,
            REFRACTORY_PERIODS,
            milliseconds,
        );
    }
    if let Ok(mut neuron) = world.get::<&mut LeakyNeuron>(entity) {
        heading(ui, chrome, "Potentials", 0.0);
        let threshold = parameter(
            ui,
            chrome,
            Setting::Threshold,
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
            Setting::Reset,
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
            Setting::RestingPotential,
            &mut neuron.resting_potential,
            POTENTIALS,
            millivolts,
        );
    }
    if let (Ok(mut neuron), Ok(mut leak)) = (
        world.get::<&mut LeakyNeuron>(entity),
        world.get::<&mut LeakCurrent>(entity),
    ) {
        heading(ui, chrome, "Membrane", 0.0);
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Resistance,
            &mut leak.resistance,
            RESISTANCES,
            |ohms| format!("{:.0} M\u{3a9}", ohms * 1e-6),
        );
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Capacitance,
            &mut neuron.capacitance,
            CAPACITANCES,
            |farads| format!("{:.0} pF", farads * 1e12),
        );
        reading(
            ui,
            chrome,
            Setting::TimeConstant.label(),
            milliseconds(leak.resistance * neuron.capacitance),
            12.5,
        );
        description(ui, chrome, Setting::TimeConstant.description());
        ui.add_space(8.0);
    }
    if let Ok(mut adaptation) = world.get::<&mut AdaptationCurrent>(entity) {
        heading(ui, chrome, "Adaptation", 0.0);
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Adaptation,
            &mut adaptation.adaptation,
            ADAPTATIONS,
            |siemens| format!("{:.1} nS", siemens * 1e9),
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::AdaptationTimeConstant,
            &mut adaptation.time_constant,
            ADAPTATION_TIME_CONSTANTS,
            milliseconds,
        );
    }
    changed
}

fn parameters(ui: &mut Ui, chrome: &Chrome, world: &World, entity: Entity) -> bool {
    let mut changed = false;
    let hertz = |value: f64| format!("{value:.0} Hz");
    if world.get::<&LeakyNeuron>(entity).is_ok() {
        changed |= neuron_parameters(ui, chrome, world, entity);
    }
    if let Ok(mut range) = world.get::<&mut VoltmeterRange>(entity) {
        heading(ui, chrome, "Parameters", 0.0);
        let shown = |value: f64| format!("{value:.0} mV").replace('-', "\u{2212}");
        let highest = parameter(
            ui,
            chrome,
            Setting::HighestVoltage,
            &mut range.maximum,
            PLOTTED_VOLTAGES,
            shown,
        );
        if highest {
            range.maximum = range.maximum.max(range.minimum + PLOTTED_VOLTAGE_GAP);
        }
        let lowest = parameter(
            ui,
            chrome,
            Setting::LowestVoltage,
            &mut range.minimum,
            PLOTTED_VOLTAGES,
            shown,
        );
        if lowest {
            range.minimum = range.minimum.min(range.maximum - PLOTTED_VOLTAGE_GAP);
        }
        changed |= highest || lowest;
    }
    if let Ok(mut synapse) = world.get::<&mut CurrentSynapse>(entity) {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::MaximumCurrent,
            &mut synapse.maximum_current,
            SYNAPSE_CURRENTS,
            nanoamperes,
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::SynapseTimeConstant,
            &mut synapse.tau,
            SYNAPSE_TIME_CONSTANTS,
            milliseconds,
        );
        changed |= parameter(
            ui,
            chrome,
            Setting::Delay,
            &mut synapse.delay,
            DELAYS,
            milliseconds,
        );
    }
    if let Ok(mut clamp) = world.get::<&mut CurrentClamp>(entity) {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Current,
            &mut clamp.current_output,
            CURRENTS,
            nanoamperes,
        );
    }
    if world.get::<&RegularSpikeGenerator>(entity).is_ok() {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= type_switch(ui, chrome, world, entity);
    }
    if let Ok(mut generator) = world.get::<&mut RegularSpikeGenerator>(entity) {
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Frequency,
            &mut generator.frequency,
            SPIKE_RATES,
            hertz,
        );
    }
    if world.get::<&PoissonGenerator>(entity).is_ok() {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= type_switch(ui, chrome, world, entity);
    }
    if let Ok(mut generator) = world.get::<&mut PoissonGenerator>(entity) {
        changed |= logarithmic_parameter(
            ui,
            chrome,
            Setting::Rate,
            &mut generator.rate,
            SPIKE_RATES,
            hertz,
        );
    }
    if let Ok(mut annotation) = world.get::<&mut Annotation>(entity) {
        heading(ui, chrome, "Text", 0.0);
        changed |= ui
            .add(
                TextEdit::multiline(&mut annotation.text)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .margin(Margin::same(8)),
            )
            .changed();
        ui.add_space(8.0);
    }
    if let Some(ending) = axon_ending(world, entity) {
        heading(ui, chrome, "Parameters", 0.0);
        changed |= parameter(
            ui,
            chrome,
            Setting::Strength,
            &mut world.get::<&mut Connection>(ending).unwrap().strength,
            0.0..=MAX_ENDING_STRENGTH,
            |strength| format!("{strength:.2}"),
        );
    }
    if let Ok(compartment) = world.get::<&Compartment>(entity) {
        heading(ui, chrome, "Gating", 0.0);
        reading(ui, chrome, "m", format!("{:.2}", compartment.m), 12.5);
        reading(ui, chrome, "n", format!("{:.2}", compartment.n), 12.5);
        reading(ui, chrome, "h", format!("{:.2}", compartment.h), 12.5);
    }
    changed
}
