use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Id, Order, Painter, Rect, Response, Sense, Ui,
    Vec2, WidgetInfo, WidgetType,
};

use crate::editor::Editor;
use crate::elements::Element;
use crate::rendering::Theme;
use crate::tools::{Tool, ToolGroup};

use super::icons::{paint_icon, paint_tool, Icon};
use super::style::{Chrome, MARGIN};
use super::widgets::{heading, paint_flyout_marker, separator, switch, ROW_INSET, ROW_RADIUS};

pub const RAIL_WIDTH: f32 = 64.0;
pub const POPOVER_GAP: f32 = 10.0;
const RAIL_PADDING: i8 = 6;
const ITEM_HEIGHT: f32 = 52.0;
const ITEM_RADIUS: u8 = 10;
const ITEM_ICON: f32 = 24.0;
const FLYOUT_WIDTH: f32 = 272.0;
const FLYOUT_PADDING: i8 = 8;
const OPTION_GLYPH: f32 = 34.0;
const OPTION_INSET: f32 = 8.0;

#[derive(Clone, Copy)]
enum Slot {
    Tool(Tool),
    Group(ToolGroup),
}

const SECTIONS: [&[Slot]; 2] = [
    &[
        Slot::Tool(Tool::Select),
        Slot::Tool(Tool::Stimulate),
        Slot::Tool(Tool::Erase),
    ],
    &[
        Slot::Group(ToolGroup::Neurons),
        Slot::Group(ToolGroup::Inputs),
        Slot::Group(ToolGroup::Connections),
        Slot::Tool(Tool::Voltmeter),
        Slot::Tool(Tool::Place(Element::Note)),
    ],
];

pub struct RailResponse {
    pub rect: Rect,
    pub menu: Response,
}

#[derive(Default)]
pub struct Rail {
    flyout: Option<ToolGroup>,
    recent: Vec<Tool>,
}

impl Rail {
    pub fn close(&mut self) {
        self.flyout = None;
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
    }

    pub fn show(
        &mut self,
        context: &Context,
        theme: Theme,
        menu_open: bool,
        editor: &mut Editor,
    ) -> RailResponse {
        let chrome = Chrome::of(theme);
        let height = context.content_rect().height() - 2.0 * MARGIN;
        let mut toggled = None;
        let mut groups = Vec::new();
        let inner = egui::Area::new(Id::new("rail"))
            .anchor(Align2::LEFT_TOP, [MARGIN, MARGIN])
            .show(context, |ui| {
                chrome
                    .panel_frame(RAIL_PADDING)
                    .show(ui, |ui| {
                        ui.set_width(RAIL_WIDTH - 2.0 * RAIL_PADDING as f32);
                        ui.set_min_height(height - 2.0 * RAIL_PADDING as f32 - 2.0);
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
                            for slot in section {
                                match *slot {
                                    Slot::Tool(tool) => {
                                        let response = tool_item(
                                            ui,
                                            chrome,
                                            theme,
                                            tool,
                                            tool.name(),
                                            editor.tool == tool,
                                            false,
                                        )
                                        .on_hover_text(tool.description());
                                        if response.clicked() {
                                            self.choose(editor, tool);
                                        }
                                    }
                                    Slot::Group(group) => {
                                        let shown = self.shown_for(group, editor.tool);
                                        let on = group.tools().contains(&editor.tool);
                                        let response = tool_item(
                                            ui,
                                            chrome,
                                            theme,
                                            shown,
                                            group.label(),
                                            on,
                                            true,
                                        );
                                        groups.push((group, response.rect));
                                        if response.clicked() {
                                            toggled = Some(group);
                                        }
                                    }
                                }
                            }
                        }
                        menu
                    })
                    .inner
            });
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
            menu: inner.inner,
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
                    for &tool in group.tools() {
                        if option(ui, chrome, theme, tool, editor.tool == tool).clicked() {
                            self.choose(editor, tool);
                        }
                    }
                    if group == ToolGroup::Neurons {
                        separator(ui, chrome);
                        switch(
                            ui,
                            chrome,
                            "New neurons get dendrites",
                            &mut editor.dendrites,
                        );
                    }
                });
            })
            .response
            .rect
    }
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
    has_flyout: bool,
) -> Response {
    item(ui, chrome, label, on, has_flyout, |painter, rect, color| {
        paint_tool(painter, tool, rect, theme, color)
    })
}

fn option(ui: &mut Ui, chrome: &Chrome, theme: Theme, tool: Tool, on: bool) -> Response {
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
    paint_tool(painter, tool, glyph, theme, chrome.text);
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
