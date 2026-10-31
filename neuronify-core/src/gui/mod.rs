mod icons;
mod inspector;
mod playback;
mod rail;
mod sheet;
mod style;
#[cfg(test)]
mod tests;
mod widgets;

use egui::{
    pos2, vec2, Align, Align2, Color32, Context, Id, Key, KeyboardShortcut, Layout, Modifiers,
    Order, Pos2, Rect, Response, RichText, Sense, Ui, Vec2, WidgetInfo, WidgetType,
};
use hecs::World;

use crate::components::{NeuronDisplay, Setting};
use crate::editor::Editor;
use crate::rendering::Theme;
use crate::tools::Tool;
use icons::Icon;
use inspector::{Inspector, Placement};
use playback::BAR_HEIGHT;
use rail::{Rail, POPOVER_GAP, RAIL_WIDTH, TOOL_BAR_HEIGHT, TOOL_BAR_MARGIN};
use style::{Chrome, MARGIN, PANEL_RADIUS};
use widgets::{icon_row, segmented, separator, switch};

const FAB_SIZE: f32 = 46.0;
const FAB_GAP: f32 = 8.0;
const ICON_SIZE: f32 = 22.0;
const DISABLED_OPACITY: f32 = 0.45;
const MENU_WIDTH: f32 = 232.0;
const MENU_PADDING: i8 = 6;
const MENU_ROW_HEIGHT: f32 = 34.0;
const COMPACT_WIDTH: f32 = 640.0;
const COMPACT_HEIGHT: f32 = 560.0;
const HINT_GAP: f32 = 20.0;
const HINT_PADDING: i8 = 8;
const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);
const REDO_Y: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

#[derive(Debug, PartialEq)]
pub enum MenuChoice {
    Open,
    Save,
    Share,
    Examples,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Saving {
    Save,
    SaveOrShare,
    Share,
}

pub struct Controls<'a> {
    pub editor: &'a mut Editor,
    pub world: &'a mut World,
    pub theme: &'a mut Theme,
    pub time: f64,
    pub running: &'a mut bool,
    pub speed: &'a mut f64,
    pub fps: f64,
    pub saving: Saving,
}

#[derive(Default)]
pub struct Gui {
    rail: Rail,
    inspector: Inspector,
    menu_open: bool,
    playback_open: bool,
}

impl Gui {
    pub fn rail_shown(&self) -> bool {
        self.rail.is_shown()
    }

    pub fn show(&mut self, context: &Context, controls: Controls) -> Option<MenuChoice> {
        if controls.editor.tool.edits() {
            history_shortcuts(context, controls.editor, controls.world);
        }
        self.rail.shortcuts(context, controls.editor);
        if is_compact(context.content_rect()) {
            self.compact(context, controls)
        } else {
            self.wide(context, controls)
        }
    }

    fn wide(&mut self, context: &Context, mut controls: Controls) -> Option<MenuChoice> {
        let theme = *controls.theme;
        let chrome = Chrome::of(theme);
        let editing = controls.editor.tool.edits();
        let mut choice = None;
        if corner_buttons(context, theme) {
            choice = Some(MenuChoice::Examples);
        }
        if editing {
            floating(
                context,
                "history_buttons",
                Align2::RIGHT_BOTTOM,
                vec2(-(MARGIN + FAB_SIZE + FAB_GAP), -MARGIN),
                |ui| history_buttons(ui, chrome, controls.editor, controls.world),
            );
        }
        let rail = self
            .rail
            .show(context, theme, self.menu_open, controls.editor);
        let menu_button = rail.menu.map(|menu| {
            let position = pos2(
                rail.rect.right() + POPOVER_GAP,
                menu.rect.top() - MENU_PADDING as f32,
            );
            (menu, position)
        });
        self.inspector.show(
            context,
            theme,
            controls.world,
            controls.editor,
            controls.time,
            Placement::Panel,
        );
        let shift = if self.rail.is_shown() {
            (MARGIN + RAIL_WIDTH) / 2.0
        } else {
            0.0
        };
        playback::bar(
            context,
            theme,
            Align2::CENTER_BOTTOM,
            vec2(shift, -MARGIN),
            controls.running,
            controls.speed,
            controls.time,
        );
        let Some((menu_button, menu_position)) = menu_button else {
            self.menu_open = false;
            return choice;
        };
        let picked = self.menu(
            context,
            &menu_button,
            (Align2::LEFT_TOP, menu_position),
            &mut controls,
        );
        choice.or(picked)
    }

    fn compact(&mut self, context: &Context, mut controls: Controls) -> Option<MenuChoice> {
        if controls.editor.selection.is_some() && !controls.editor.tool.edits() {
            controls.editor.tool = Tool::Edit;
        }
        let theme = *controls.theme;
        let chrome = Chrome::of(theme);
        let editing = controls.editor.tool.edits();
        let mut choice = None;
        let back = floating(
            context,
            "back_button",
            Align2::LEFT_TOP,
            vec2(MARGIN, MARGIN),
            |ui| fab(ui, chrome, Icon::Back, "Examples", false),
        );
        if back.clicked() {
            choice = Some(MenuChoice::Examples);
        }
        let more_inset = if editing {
            MARGIN + FAB_SIZE + FAB_GAP
        } else {
            MARGIN
        };
        let open = self.menu_open;
        let more = floating(
            context,
            "more_button",
            Align2::RIGHT_TOP,
            vec2(-more_inset, MARGIN),
            |ui| fab(ui, chrome, Icon::More, "More", open),
        );
        if editing {
            let done = floating(
                context,
                "done_button",
                Align2::RIGHT_TOP,
                vec2(-MARGIN, MARGIN),
                |ui| fab(ui, chrome, Icon::Check, "Done", true),
            );
            if done.clicked() {
                controls.editor.selection = None;
                controls.editor.tool = Tool::View;
                self.rail.close();
            }
            floating(
                context,
                "history_buttons",
                Align2::RIGHT_TOP,
                vec2(-(more_inset + FAB_SIZE + FAB_GAP), MARGIN),
                |ui| history_buttons(ui, chrome, controls.editor, controls.world),
            );
        }
        self.rail
            .bar(context, theme, controls.editor, &mut self.playback_open);
        if self.playback_open {
            playback::bar(
                context,
                theme,
                Align2::CENTER_BOTTOM,
                vec2(0.0, -(TOOL_BAR_MARGIN + TOOL_BAR_HEIGHT + FAB_GAP)),
                controls.running,
                controls.speed,
                controls.time,
            );
        }
        self.inspector.show(
            context,
            theme,
            controls.world,
            controls.editor,
            controls.time,
            Placement::Sheet,
        );
        let position = pos2(more.rect.right(), more.rect.bottom() + FAB_GAP);
        let picked = self.menu(context, &more, (Align2::RIGHT_TOP, position), &mut controls);
        choice.or(picked)
    }

    fn menu(
        &mut self,
        context: &Context,
        button: &Response,
        (pivot, position): (Align2, Pos2),
        controls: &mut Controls,
    ) -> Option<MenuChoice> {
        if button.clicked() {
            self.menu_open = !self.menu_open;
        }
        if !self.menu_open {
            return None;
        }
        let (picked, rect) = menu(context, pivot, position, controls);
        let pressed_elsewhere = context.input(|input| {
            input.pointer.any_pressed()
                && input.pointer.interact_pos().is_some_and(|position| {
                    !rect.contains(position) && !button.rect.contains(position)
                })
        });
        if picked.is_some() || pressed_elsewhere {
            self.menu_open = false;
        }
        picked
    }
}

pub fn is_compact(screen: Rect) -> bool {
    screen.width() < COMPACT_WIDTH || screen.height() < COMPACT_HEIGHT
}

pub fn canvas_area(screen: Rect, rail_shown: bool) -> Rect {
    let mut area = screen;
    if is_compact(screen) {
        area.min.y += MARGIN + FAB_SIZE;
        area.max.y -= TOOL_BAR_MARGIN + TOOL_BAR_HEIGHT;
    } else {
        if rail_shown {
            area.min.x += MARGIN + RAIL_WIDTH;
        }
        area.max.y -= MARGIN + BAR_HEIGHT;
    }
    area
}

pub fn load_error(context: &Context, error: &str) {
    egui::Window::new("Could not load file").show(context, |ui| {
        ui.label(error);
    });
}

pub fn connection_hint(context: &Context, theme: Theme, tip: Pos2, shown: f32) {
    let chrome = Chrome::of(theme);
    egui::Area::new(Id::new("connection_hint"))
        .order(Order::Tooltip)
        .interactable(false)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(tip - vec2(0.0, HINT_GAP))
        .show(context, |ui| {
            ui.multiply_opacity(shown);
            chrome.panel_frame(HINT_PADDING).show(ui, |ui| {
                ui.add(
                    egui::Label::new(RichText::new("Drag to connect").color(chrome.text)).extend(),
                );
            });
        });
}

fn fab(ui: &mut Ui, chrome: &Chrome, icon: Icon, label: &str, on: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(FAB_SIZE), Sense::click());
    let (fill, color) = if on {
        (chrome.accent, chrome.on_accent)
    } else if response.hovered() {
        (chrome.hovered, chrome.text)
    } else {
        (chrome.panel, chrome.text)
    };
    paint_fab(ui, chrome, icon, rect, fill, color);
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, on, label));
    response.on_hover_text(label)
}

fn action_fab(
    ui: &mut Ui,
    chrome: &Chrome,
    icon: Icon,
    label: &str,
    shortcut: &KeyboardShortcut,
    enabled: bool,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(FAB_SIZE), sense);
    let (fill, color) = if !enabled {
        (
            chrome.panel,
            chrome.subtext.gamma_multiply(DISABLED_OPACITY),
        )
    } else if response.hovered() {
        (chrome.hovered, chrome.text)
    } else {
        (chrome.panel, chrome.text)
    };
    paint_fab(ui, chrome, icon, rect, fill, color);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    let shortcut = ui.ctx().format_shortcut(shortcut);
    response.on_hover_text(format!("{label} ({shortcut})"))
}

fn paint_fab(ui: &Ui, chrome: &Chrome, icon: Icon, rect: Rect, fill: Color32, color: Color32) {
    chrome.paint_floating(ui.painter(), rect, PANEL_RADIUS, fill);
    icons::paint_icon(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), Vec2::splat(ICON_SIZE)),
        color,
    );
}

fn history_buttons(
    ui: &mut Ui,
    chrome: &Chrome,
    editor: &mut Editor,
    world: &mut World,
) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = FAB_GAP;
        if action_fab(ui, chrome, Icon::Undo, "Undo", &UNDO, editor.can_undo()).clicked() {
            editor.undo(world);
        }
        if action_fab(ui, chrome, Icon::Redo, "Redo", &REDO, editor.can_redo()).clicked() {
            editor.redo(world);
        }
    })
    .response
}

fn history_shortcuts(context: &Context, editor: &mut Editor, world: &mut World) {
    if context.egui_wants_keyboard_input() {
        return;
    }
    let redo =
        context.input_mut(|input| input.consume_shortcut(&REDO) || input.consume_shortcut(&REDO_Y));
    if redo {
        editor.redo(world);
    } else if context.input_mut(|input| input.consume_shortcut(&UNDO)) {
        editor.undo(world);
    }
}

fn floating(
    context: &Context,
    id: &str,
    anchor: Align2,
    offset: Vec2,
    contents: impl FnOnce(&mut Ui) -> Response,
) -> Response {
    egui::Area::new(Id::new(id))
        .anchor(anchor, offset)
        .show(context, contents)
        .inner
}

fn corner_buttons(context: &Context, theme: Theme) -> bool {
    let chrome = Chrome::of(theme);
    let mut examples_clicked = false;
    egui::Area::new("corner_buttons".into())
        .anchor(Align2::RIGHT_BOTTOM, [-MARGIN, -MARGIN])
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = FAB_GAP;
                examples_clicked = fab(ui, chrome, Icon::Grid, "Examples", false).clicked();
            });
        });
    examples_clicked
}

fn menu(
    context: &Context,
    pivot: Align2,
    position: Pos2,
    controls: &mut Controls,
) -> (Option<MenuChoice>, Rect) {
    let chrome = Chrome::of(*controls.theme);
    let saving = controls.saving;
    let mut picked = None;
    let rect = egui::Area::new(Id::new("menu"))
        .order(Order::Foreground)
        .pivot(pivot)
        .fixed_pos(position)
        .show(context, |ui| {
            chrome.panel_frame(MENU_PADDING).show(ui, |ui| {
                ui.set_width(MENU_WIDTH);
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                if icon_row(ui, chrome, Icon::Folder, "Open…").clicked() {
                    picked = Some(MenuChoice::Open);
                }
                if saving != Saving::Share && icon_row(ui, chrome, Icon::Save, "Save…").clicked()
                {
                    picked = Some(MenuChoice::Save);
                }
                if saving != Saving::Save && icon_row(ui, chrome, Icon::Share, "Share…").clicked()
                {
                    picked = Some(MenuChoice::Share);
                }
                if icon_row(ui, chrome, Icon::Grid, "Examples").clicked() {
                    picked = Some(MenuChoice::Examples);
                }
                separator(ui, chrome);
                labelled_row(ui, chrome, "Theme", |ui| {
                    segmented(
                        ui,
                        chrome,
                        controls.theme,
                        &[(Theme::Dark, "Dark"), (Theme::Light, "Light")],
                    )
                });
                let mut display = NeuronDisplay::of(controls.world);
                let voltage = switch(ui, chrome, "Show voltage", &mut display.voltage);
                let firing_rate = switch(ui, chrome, "Show firing rate", &mut display.firing_rate);
                if voltage.changed() || firing_rate.changed() {
                    display.set_in(controls.world);
                }
                labelled_row(ui, chrome, "Frame rate", |ui| {
                    ui.label(
                        RichText::new(format!("{:.0} fps", controls.fps))
                            .size(13.0)
                            .color(chrome.subtext),
                    );
                });
            });
        })
        .response
        .rect;
    (picked, rect)
}

fn labelled_row(ui: &mut Ui, chrome: &Chrome, label: &str, value: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), MENU_ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(label).size(13.5).color(chrome.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(4.0);
                value(ui);
            });
        },
    );
}
