use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{
    pos2, vec2, Align, Align2, Color32, Context, Event, FontId, Galley, Id, Key, Order, Painter,
    Rect, Response, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType,
};

use crate::editor::Editor;
use crate::elements::Element;
use crate::rendering::Theme;
use crate::tools::{Tool, ToolGroup};

use super::icons::{paint_icon, paint_morphology, paint_tool, Icon};
use super::sheet::{self, Sheet, HANDLE_HEIGHT};
use super::style::{Chrome, MARGIN, PANEL_RADIUS};
use super::widgets::{heading, paint_flyout_marker, separator, ROW_INSET, ROW_RADIUS};

pub const RAIL_WIDTH: f32 = 64.0;
pub const POPOVER_GAP: f32 = 10.0;
pub const TOOL_BAR_HEIGHT: f32 = ITEM_HEIGHT + 2.0 * RAIL_PADDING as f32;
pub const TOOL_BAR_MARGIN: f32 = 8.0;
const TOOL_BAR_SLOTS: usize = 5;
const TILE_HEIGHT: f32 = 78.0;
const TILE_GLYPH: f32 = 34.0;
const TILES_PER_ROW: usize = 4;
const SHEET_HEIGHT: f32 = 0.8;

const RAIL_PADDING: i8 = 6;
const HIDE_HEIGHT: f32 = 22.0;
const HIDE_ICON: f32 = 14.0;
const HIDE_OPACITY: f32 = 0.6;
const SHOW_SIZE: f32 = 40.0;
const ITEM_HEIGHT: f32 = 52.0;
const ITEM_RADIUS: u8 = 10;
const ITEM_ICON: f32 = 24.0;
const FLYOUT_WIDTH: f32 = 272.0;
const FLYOUT_PADDING: i8 = 8;
const OPTION_GLYPH: f32 = 34.0;
const OPTION_INSET: f32 = 8.0;
const CARD_SPACING: f32 = 6.0;
const CARD_GAP: f32 = 6.0;
const CARD_PICTURE: Vec2 = vec2(80.0, 40.0);
const INPUT_HINT: &str = "Press and drag to place an input and connect it to a neuron.";
const MORPHOLOGIES: [(bool, &str, &str); 2] = [
    (
        false,
        "Point neurons",
        "Drag from a neuron to connect it to another.",
    ),
    (
        true,
        "Compartmental neurons",
        "Drag from a neuron or dendrite to grow an axon.",
    ),
];

#[derive(Clone, Copy)]
enum Slot {
    Tool(Tool),
    Group(ToolGroup),
}

const SECTIONS: [&[(Slot, Key)]; 2] = [
    &[
        (Slot::Tool(Tool::View), Key::H),
        (Slot::Tool(Tool::Edit), Key::V),
        (Slot::Tool(Tool::Stimulate), Key::S),
        (Slot::Tool(Tool::Erase), Key::E),
    ],
    &[
        (Slot::Group(ToolGroup::Neurons), Key::N),
        (Slot::Group(ToolGroup::Inputs), Key::I),
        (Slot::Group(ToolGroup::Connections), Key::C),
        (Slot::Group(ToolGroup::Meters), Key::M),
        (Slot::Tool(Tool::Place(Element::Note)), Key::T),
    ],
];

pub struct RailResponse {
    pub rect: Rect,
    pub menu: Option<Response>,
}

#[derive(Default)]
pub struct Rail {
    flyout: Option<ToolGroup>,
    adding: bool,
    sheet: Sheet,
    recent: Vec<Tool>,
    hidden: bool,
}

impl Rail {
    pub fn is_shown(&self) -> bool {
        !self.hidden
    }

    pub fn close(&mut self) {
        self.flyout = None;
        self.adding = false;
    }

    fn shown_for(&self, group: ToolGroup, current: Tool) -> Tool {
        if group.tools().contains(&current) {
            return current;
        }
        self.recent
            .iter()
            .rev()
            .copied()
            .find(|tool| group.tools().contains(tool))
            .unwrap_or(group.tools()[0])
    }

    fn choose(&mut self, editor: &mut Editor, tool: Tool) {
        editor.tool = tool;
        self.recent.retain(|recent| *recent != tool);
        self.recent.push(tool);
        self.flyout = None;
        self.adding = false;
    }

    pub fn shortcuts(&mut self, context: &Context, editor: &mut Editor) {
        if context.egui_wants_keyboard_input() {
            return;
        }
        let pressed = context.input(|input| {
            if input.pointer.any_down() {
                return None;
            }
            input.events.iter().find_map(|event| match *event {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } if modifiers.is_none() => SECTIONS
                    .into_iter()
                    .flatten()
                    .find(|(_, shortcut)| *shortcut == key),
                _ => None,
            })
        });
        let Some(&(slot, _)) = pressed else {
            return;
        };
        let tool = match slot {
            Slot::Tool(tool) => tool,
            Slot::Group(group) => {
                let tools = group.tools();
                match tools.iter().position(|tool| *tool == editor.tool) {
                    Some(index) => tools[(index + 1) % tools.len()],
                    None => self.shown_for(group, editor.tool),
                }
            }
        };
        self.choose(editor, tool);
    }

    pub fn show(
        &mut self,
        context: &Context,
        theme: Theme,
        menu_open: bool,
        editor: &mut Editor,
    ) -> RailResponse {
        let chrome = Chrome::of(theme);
        if self.hidden {
            let shown = egui::Area::new(Id::new("show_rail"))
                .anchor(Align2::LEFT_BOTTOM, [MARGIN, -MARGIN])
                .show(context, |ui| show_button(ui, chrome, theme, editor))
                .inner;
            if shown.clicked() {
                self.hidden = false;
            }
            return RailResponse {
                rect: shown.rect,
                menu: None,
            };
        }
        let height = context.content_rect().height() - 2.0 * MARGIN;
        let mut hide = false;
        let mut toggled = None;
        let mut groups = Vec::new();
        let inner = egui::Area::new(Id::new("rail"))
            .anchor(Align2::LEFT_TOP, [MARGIN, MARGIN])
            .show(context, |ui| {
                chrome
                    .panel_frame(RAIL_PADDING)
                    .show(ui, |ui| {
                        let inner_height = height - 2.0 * RAIL_PADDING as f32 - 2.0;
                        ui.set_width(RAIL_WIDTH - 2.0 * RAIL_PADDING as f32);
                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                        let menu = item(
                            ui,
                            chrome,
                            "Menu",
                            menu_open,
                            false,
                            |painter, rect, color| paint_icon(painter, Icon::Menu, rect, color),
                        );
                        for section in SECTIONS {
                            separator(ui, chrome);
                            for &(slot, key) in section {
                                match slot {
                                    Slot::Tool(tool) => {
                                        let response = tool_item(
                                            ui,
                                            chrome,
                                            theme,
                                            tool,
                                            tool.name(),
                                            editor.tool == tool,
                                        )
                                        .on_hover_text(format!(
                                            "{} ({})\n{}",
                                            tool.name(),
                                            key.name(),
                                            tool.description()
                                        ));
                                        if response.clicked() {
                                            self.choose(editor, tool);
                                        }
                                    }
                                    Slot::Group(group) => {
                                        let shown = self.shown_for(group, editor.tool);
                                        let on = group.tools().contains(&editor.tool);
                                        let compartments = editor.compartments;
                                        let response = item(
                                            ui,
                                            chrome,
                                            group.label(),
                                            on,
                                            true,
                                            |painter, rect, color| {
                                                paint_tool(
                                                    painter,
                                                    shown,
                                                    rect,
                                                    theme,
                                                    color,
                                                    compartments,
                                                )
                                            },
                                        )
                                        .on_hover_text(format!(
                                            "{} ({})",
                                            group.label(),
                                            key.name()
                                        ));
                                        groups.push((group, response.rect));
                                        if response.clicked() {
                                            toggled = Some(group);
                                        }
                                    }
                                }
                            }
                        }
                        let spacing = ui.spacing().item_spacing.y;
                        let used = ui.min_rect().height();
                        ui.add_space((inner_height - used - HIDE_HEIGHT - spacing).max(0.0));
                        hide = hide_button(ui, chrome).clicked();
                        menu
                    })
                    .inner
            });
        if hide {
            self.hidden = true;
            self.flyout = None;
        }
        let rail = inner.response.rect;
        if let Some(group) = toggled {
            self.flyout = (self.flyout != Some(group)).then_some(group);
        }
        if inner.inner.clicked() {
            self.flyout = None;
        }
        if let Some(group) = self.flyout {
            let anchor = groups
                .iter()
                .find(|(candidate, _)| *candidate == group)
                .map_or(rail, |(_, rect)| *rect);
            let flyout = self.flyout(context, theme, group, editor, rail, anchor);
            let pressed_elsewhere = context.input(|input| {
                input.pointer.any_pressed()
                    && input.pointer.interact_pos().is_some_and(|position| {
                        !flyout.contains(position) && !rail.contains(position)
                    })
            });
            if pressed_elsewhere {
                self.flyout = None;
            }
        }
        RailResponse {
            rect: rail,
            menu: Some(inner.inner),
        }
    }

    fn flyout(
        &mut self,
        context: &Context,
        theme: Theme,
        group: ToolGroup,
        editor: &mut Editor,
        rail: Rect,
        anchor: Rect,
    ) -> Rect {
        let chrome = Chrome::of(theme);
        egui::Area::new(Id::new("flyout"))
            .order(Order::Foreground)
            .fixed_pos(pos2(
                rail.right() + POPOVER_GAP,
                anchor.top() - FLYOUT_PADDING as f32,
            ))
            .show(context, |ui| {
                chrome.panel_frame(FLYOUT_PADDING).show(ui, |ui| {
                    ui.set_width(FLYOUT_WIDTH);
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    heading(ui, chrome, group.label(), ROW_INSET);
                    match group {
                        ToolGroup::Neurons => morphology_picker(
                            ui,
                            chrome,
                            theme,
                            &mut editor.compartments,
                            ROW_INSET,
                        ),
                        ToolGroup::Inputs => hint(ui, chrome, INPUT_HINT, ROW_INSET),
                        ToolGroup::Connections | ToolGroup::Meters => {}
                    }
                    for &tool in group.tools() {
                        let on = editor.tool == tool;
                        if option(ui, chrome, theme, tool, on, editor.compartments).clicked() {
                            self.choose(editor, tool);
                        }
                    }
                });
            })
            .response
            .rect
    }
}

impl Rail {
    pub fn bar(
        &mut self,
        context: &Context,
        theme: Theme,
        editor: &mut Editor,
        playback_open: &mut bool,
    ) {
        let chrome = Chrome::of(theme);
        let width = context.content_rect().width() - 2.0 * TOOL_BAR_MARGIN;
        let mut add = None;
        egui::Area::new(Id::new("tool_bar"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -TOOL_BAR_MARGIN])
            .show(context, |ui| {
                chrome
                    .panel_frame(RAIL_PADDING)
                    .corner_radius(PANEL_RADIUS + 6)
                    .show(ui, |ui| {
                        ui.set_width(width - 2.0 * RAIL_PADDING as f32);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            let slot = vec2(
                                (ui.available_width() - 2.0 * (TOOL_BAR_SLOTS - 1) as f32)
                                    / TOOL_BAR_SLOTS as f32,
                                ITEM_HEIGHT,
                            );
                            for tool in [Tool::Edit, Tool::Stimulate, Tool::Erase] {
                                let on = editor.tool == tool;
                                let response = ui
                                    .allocate_ui(slot, |ui| {
                                        tool_item(ui, chrome, theme, tool, tool.name(), on)
                                    })
                                    .inner;
                                if response.clicked() {
                                    self.choose(editor, tool);
                                }
                            }
                            let placing = ![Tool::View, Tool::Edit, Tool::Stimulate, Tool::Erase]
                                .contains(&editor.tool);
                            let response = ui
                                .allocate_ui(slot, |ui| {
                                    item(
                                        ui,
                                        chrome,
                                        "Add",
                                        placing || self.adding,
                                        false,
                                        |painter, rect, color| {
                                            if placing {
                                                paint_tool(
                                                    painter,
                                                    editor.tool,
                                                    rect,
                                                    theme,
                                                    color,
                                                    editor.compartments,
                                                );
                                            } else {
                                                paint_icon(painter, Icon::Plus, rect, color);
                                            }
                                        },
                                    )
                                })
                                .inner;
                            if response.clicked() {
                                self.adding = !self.adding;
                            }
                            add = Some(response.rect);
                            let response = ui
                                .allocate_ui(slot, |ui| {
                                    item(
                                        ui,
                                        chrome,
                                        "Playback",
                                        *playback_open,
                                        false,
                                        |painter, rect, color| {
                                            paint_icon(painter, Icon::Playback, rect, color)
                                        },
                                    )
                                })
                                .inner;
                            if response.clicked() {
                                *playback_open = !*playback_open;
                            }
                        });
                    });
            });
        if self.adding {
            let sheet = self.add_sheet(context, theme, editor);
            let pressed_elsewhere = context.input(|input| {
                input.pointer.any_pressed()
                    && input.pointer.interact_pos().is_some_and(|position| {
                        !sheet.contains(position) && !add.is_some_and(|add| add.contains(position))
                    })
            });
            if pressed_elsewhere {
                self.adding = false;
            }
        }
    }

    fn add_sheet(&mut self, context: &Context, theme: Theme, editor: &mut Editor) -> Rect {
        let chrome = Chrome::of(theme);
        let width = context.content_rect().width();
        let padding = 16;
        let id = Id::new("add_sheet");
        let detents =
            [context.content_rect().height() * SHEET_HEIGHT + HANDLE_HEIGHT + padding as f32];
        let height = self.sheet.height(context, id, &detents);
        let (laid_out, dropped) = sheet::layout(height, &detents);
        egui::Area::new(id)
            .order(Order::Foreground)
            .anchor(Align2::LEFT_BOTTOM, [0.0, dropped])
            .show(context, |ui| {
                chrome.sheet_frame(padding).show(ui, |ui| {
                    ui.set_width(width - 2.0 * padding as f32);
                    ui.set_min_height(laid_out - padding as f32);
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    if self.sheet.handle(ui, chrome, height, &detents) {
                        self.adding = false;
                    }
                    let room = laid_out - HANDLE_HEIGHT - padding as f32;
                    ScrollArea::vertical()
                        .max_height(room)
                        .min_scrolled_height(room)
                        .show(ui, |ui| {
                            let groups = [
                                ToolGroup::Neurons,
                                ToolGroup::Inputs,
                                ToolGroup::Connections,
                            ]
                            .map(|group| (Some(group), group.label(), group.tools()));
                            let measuring: Vec<Tool> = ToolGroup::Meters
                                .tools()
                                .iter()
                                .copied()
                                .chain([Tool::Place(Element::Note)])
                                .collect();
                            for (group, label, tools) in groups.into_iter().chain([(
                                None,
                                "Measure and annotate",
                                &measuring[..],
                            )]) {
                                heading(ui, chrome, label, 0.0);
                                match group {
                                    Some(ToolGroup::Neurons) => morphology_picker(
                                        ui,
                                        chrome,
                                        theme,
                                        &mut editor.compartments,
                                        0.0,
                                    ),
                                    Some(ToolGroup::Inputs) => hint(ui, chrome, INPUT_HINT, 0.0),
                                    _ => {}
                                }
                                for row in tools.chunks(TILES_PER_ROW) {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 0.0;
                                        let size = vec2(
                                            ui.available_width() / TILES_PER_ROW as f32,
                                            TILE_HEIGHT,
                                        );
                                        for &tool in row {
                                            if tile(
                                                ui,
                                                chrome,
                                                theme,
                                                tool,
                                                size,
                                                editor.tool == tool,
                                                editor.compartments,
                                            )
                                            .clicked()
                                            {
                                                self.choose(editor, tool);
                                                editor.selection = None;
                                            }
                                        }
                                    });
                                }
                            }
                        });
                });
            })
            .response
            .rect
    }
}

fn morphology_picker(
    ui: &mut Ui,
    chrome: &Chrome,
    theme: Theme,
    compartments: &mut bool,
    inset: f32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = CARD_GAP;
        let width = (ui.available_width() - CARD_GAP) / 2.0;
        let names = MORPHOLOGIES.map(|(_, label, _)| card_name(ui, chrome, label, width));
        let tallest = names.iter().map(|name| name.size().y).fold(0.0, f32::max);
        let size = vec2(width, CARD_PICTURE.y + tallest + 4.0 * CARD_SPACING);
        for ((value, _, _), name) in MORPHOLOGIES.into_iter().zip(names) {
            let on = *compartments == value;
            if morphology_card(ui, chrome, theme, value, name, size, on).clicked() {
                *compartments = value;
            }
        }
    });
    let text = MORPHOLOGIES
        .iter()
        .find(|(value, _, _)| value == compartments)
        .map_or("", |(_, _, hint)| hint);
    hint(ui, chrome, text, inset);
}

fn hint(ui: &mut Ui, chrome: &Chrome, text: &str, inset: f32) {
    let width = ui.available_width();
    let galley = ui.painter().layout(
        text.to_string(),
        FontId::proportional(12.0),
        chrome.subtext,
        width - 2.0 * inset,
    );
    let (rect, _) = ui.allocate_exact_size(vec2(width, galley.size().y + 10.0), Sense::hover());
    ui.painter().galley(
        pos2(rect.left() + inset, rect.top() + 4.0),
        galley,
        chrome.subtext,
    );
}

fn card_name(ui: &Ui, chrome: &Chrome, label: &str, width: f32) -> Arc<Galley> {
    let mut name = LayoutJob::simple(
        label.to_string(),
        FontId::proportional(11.5),
        chrome.text,
        width - 2.0 * CARD_SPACING,
    );
    name.halign = Align::Center;
    ui.fonts_mut(|fonts| fonts.layout_job(name))
}

fn morphology_card(
    ui: &mut Ui,
    chrome: &Chrome,
    theme: Theme,
    compartments: bool,
    name: Arc<Galley>,
    size: Vec2,
    on: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let (fill, border) = if on {
        (chrome.selected(), chrome.accent)
    } else if response.hovered() {
        (chrome.raised, chrome.border)
    } else {
        (Color32::TRANSPARENT, chrome.border)
    };
    painter.rect(
        rect,
        ROW_RADIUS,
        fill,
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );
    let picture = Rect::from_center_size(
        pos2(
            rect.center().x,
            rect.top() + CARD_SPACING + CARD_PICTURE.y / 2.0,
        ),
        CARD_PICTURE,
    );
    paint_morphology(painter, compartments, picture, theme);
    let label = name.text().to_string();
    painter.galley(
        pos2(rect.center().x, picture.bottom() + CARD_SPACING),
        name,
        chrome.text,
    );
    response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, on, &label));
    response
}

fn tile(
    ui: &mut Ui,
    chrome: &Chrome,
    theme: Theme,
    tool: Tool,
    size: Vec2,
    on: bool,
    compartments: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    if on {
        painter.rect_filled(rect.shrink(1.0), ROW_RADIUS, chrome.selected());
    } else if response.hovered() {
        painter.rect_filled(rect.shrink(1.0), ROW_RADIUS, chrome.raised);
    }
    paint_tool(
        painter,
        tool,
        Rect::from_center_size(
            pos2(rect.center().x, rect.top() + 6.0 + TILE_GLYPH / 2.0),
            Vec2::splat(TILE_GLYPH),
        ),
        theme,
        chrome.text,
        compartments,
    );
    let mut name = LayoutJob::simple(
        tool.name().to_string(),
        FontId::proportional(11.0),
        chrome.text,
        size.x - 8.0,
    );
    name.halign = Align::Center;
    let name = ui.fonts_mut(|fonts| fonts.layout_job(name));
    painter.galley(
        pos2(rect.center().x, rect.top() + 10.0 + TILE_GLYPH),
        name,
        chrome.text,
    );
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, on, tool.name()));
    response
}

fn hide_button(ui: &mut Ui, chrome: &Chrome) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), HIDE_HEIGHT), Sense::click());
    let color = if response.hovered() {
        chrome.text
    } else {
        chrome.subtext.gamma_multiply(HIDE_OPACITY)
    };
    paint_icon(
        ui.painter(),
        Icon::Back,
        Rect::from_center_size(rect.center(), Vec2::splat(HIDE_ICON)),
        color,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Hide tools"));
    response.on_hover_text("Hide tools")
}

fn show_button(ui: &mut Ui, chrome: &Chrome, theme: Theme, editor: &Editor) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(SHOW_SIZE), Sense::click());
    let fill = if response.hovered() {
        chrome.hovered
    } else {
        chrome.panel
    };
    chrome.paint_floating(ui.painter(), rect, (SHOW_SIZE / 2.0) as u8, fill);
    paint_tool(
        ui.painter(),
        editor.tool,
        Rect::from_center_size(rect.center(), Vec2::splat(ITEM_ICON * 0.85)),
        theme,
        chrome.text,
        editor.compartments,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Show tools"));
    response.on_hover_text(format!("Show tools ({})", editor.tool.name()))
}

fn item(
    ui: &mut Ui,
    chrome: &Chrome,
    label: &str,
    on: bool,
    has_flyout: bool,
    paint: impl FnOnce(&Painter, Rect, Color32),
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), ITEM_HEIGHT), Sense::click());
    let painter = ui.painter();
    let hovered = response.hovered();
    if on {
        painter.rect_filled(rect, ITEM_RADIUS, chrome.selected());
    } else if hovered {
        painter.rect_filled(rect, ITEM_RADIUS, chrome.raised);
    }
    paint(
        painter,
        Rect::from_center_size(
            pos2(rect.center().x, rect.top() + 20.0),
            Vec2::splat(ITEM_ICON),
        ),
        if on { chrome.accent } else { chrome.text },
    );
    painter.text(
        pos2(rect.center().x, rect.bottom() - 5.0),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(10.5),
        if on || hovered {
            chrome.text
        } else {
            chrome.subtext
        },
    );
    if has_flyout {
        paint_flyout_marker(
            painter,
            pos2(rect.right() - 4.0, rect.top() + 20.0),
            chrome.subtext,
        );
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, on, label));
    response
}

fn tool_item(
    ui: &mut Ui,
    chrome: &Chrome,
    theme: Theme,
    tool: Tool,
    label: &str,
    on: bool,
) -> Response {
    item(ui, chrome, label, on, false, |painter, rect, color| {
        paint_tool(painter, tool, rect, theme, color, false)
    })
}

fn option(
    ui: &mut Ui,
    chrome: &Chrome,
    theme: Theme,
    tool: Tool,
    on: bool,
    compartments: bool,
) -> Response {
    let width = ui.available_width();
    let text_width = width - OPTION_GLYPH - 3.0 * OPTION_INSET;
    let name = ui.painter().layout_no_wrap(
        tool.name().to_string(),
        FontId::proportional(14.0),
        chrome.text,
    );
    let description = ui.painter().layout(
        tool.description().to_string(),
        FontId::proportional(12.0),
        chrome.subtext,
        text_width,
    );
    let text_height = name.size().y + 2.0 + description.size().y;
    let height = text_height.max(OPTION_GLYPH) + 2.0 * OPTION_INSET;
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let painter = ui.painter();
    if on {
        painter.rect_filled(rect, ROW_RADIUS, chrome.selected());
    } else if response.hovered() {
        painter.rect_filled(rect, ROW_RADIUS, chrome.raised);
    }
    let glyph = Rect::from_min_size(
        pos2(
            rect.left() + OPTION_INSET,
            rect.center().y - OPTION_GLYPH / 2.0,
        ),
        Vec2::splat(OPTION_GLYPH),
    );
    paint_tool(painter, tool, glyph, theme, chrome.text, compartments);
    let text_left = glyph.right() + OPTION_INSET;
    let text_top = rect.center().y - text_height / 2.0;
    let name_height = name.size().y;
    painter.galley(pos2(text_left, text_top), name, chrome.text);
    painter.galley(
        pos2(text_left, text_top + name_height + 2.0),
        description,
        chrome.subtext,
    );
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, on, tool.name()));
    response
}
