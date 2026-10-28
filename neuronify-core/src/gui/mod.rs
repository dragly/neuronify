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
    pos2, vec2, Align, Align2, Context, Id, Layout, Order, Pos2, Rect, Response, RichText, Sense,
    Ui, Vec2, WidgetInfo, WidgetType,
};
use hecs::World;

use crate::editor::Editor;
use crate::rendering::Theme;
use crate::tools::Tool;
use icons::Icon;
use inspector::{Inspector, Placement};
use playback::BAR_HEIGHT;
use rail::{Rail, POPOVER_GAP, RAIL_WIDTH, TOOL_BAR_HEIGHT, TOOL_BAR_MARGIN};
use style::{Chrome, MARGIN, PANEL_RADIUS};
use widgets::{icon_row, segmented, separator};

const FAB_SIZE: f32 = 46.0;
const FAB_GAP: f32 = 8.0;
const ICON_SIZE: f32 = 22.0;
const MENU_WIDTH: f32 = 232.0;
const MENU_PADDING: i8 = 6;
const MENU_ROW_HEIGHT: f32 = 34.0;
const COMPACT_WIDTH: f32 = 640.0;
const COMPACT_HEIGHT: f32 = 560.0;

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
    pub edit_enabled: &'a mut bool,
    pub editor: &'a mut Editor,
    pub world: &'a mut World,
    pub theme: &'a mut Theme,
    pub time: f64,
    pub running: &'a mut bool,
    pub iterations: &'a mut u32,
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
    pub fn show(&mut self, context: &Context, controls: Controls) -> Option<MenuChoice> {
        if is_compact(context.content_rect()) {
            self.compact(context, controls)
        } else {
            self.wide(context, controls)
        }
    }

    fn wide(&mut self, context: &Context, controls: Controls) -> Option<MenuChoice> {
        let theme = *controls.theme;
        let chrome = Chrome::of(theme);
        let editing = *controls.edit_enabled;
        let mut choice = None;
        if corner_buttons(context, theme, controls.edit_enabled) {
            choice = Some(MenuChoice::Examples);
        }
        let (menu_button, menu_position) = if editing {
            let rail = self
                .rail
                .show(context, theme, self.menu_open, controls.editor);
            let position = pos2(
                rail.rect.right() + POPOVER_GAP,
                rail.menu.rect.top() - MENU_PADDING as f32,
            );
            (rail.menu, position)
        } else {
            let open = self.menu_open;
            let button = floating(
                context,
                "menu_button",
                Align2::LEFT_TOP,
                vec2(MARGIN, MARGIN),
                |ui| fab(ui, chrome, Icon::Menu, "Menu", open),
            );
            let position = pos2(button.rect.left(), button.rect.bottom() + FAB_GAP);
            (button, position)
        };
        if !editing {
            self.rail.close();
        }
        self.inspector.show(
            context,
            theme,
            controls.world,
            controls.editor,
            controls.time,
            Placement::Panel,
        );
        let shift = if editing {
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
            controls.iterations,
            controls.time,
        );
        let picked = self.menu(
            context,
            &menu_button,
            (Align2::LEFT_TOP, menu_position),
            controls.theme,
            controls.fps,
            controls.saving,
        );
        choice.or(picked)
    }

    fn compact(&mut self, context: &Context, controls: Controls) -> Option<MenuChoice> {
        if controls.editor.selection.is_some() {
            *controls.edit_enabled = true;
        }
        let theme = *controls.theme;
        let chrome = Chrome::of(theme);
        let editing = *controls.edit_enabled;
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
                *controls.edit_enabled = false;
                controls.editor.selection = None;
                controls.editor.tool = Tool::Select;
                self.rail.close();
            }
        }
        self.rail.bar(
            context,
            theme,
            controls.editor,
            controls.edit_enabled,
            &mut self.playback_open,
        );
        if self.playback_open {
            playback::bar(
                context,
                theme,
                Align2::CENTER_BOTTOM,
                vec2(0.0, -(TOOL_BAR_MARGIN + TOOL_BAR_HEIGHT + FAB_GAP)),
                controls.running,
                controls.iterations,
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
        let picked = self.menu(
            context,
            &more,
            (Align2::RIGHT_TOP, position),
            controls.theme,
            controls.fps,
            controls.saving,
        );
        choice.or(picked)
    }

    fn menu(
        &mut self,
        context: &Context,
        button: &Response,
        (pivot, position): (Align2, Pos2),
        theme: &mut Theme,
        fps: f64,
        saving: Saving,
    ) -> Option<MenuChoice> {
        if button.clicked() {
            self.menu_open = !self.menu_open;
        }
        if !self.menu_open {
            return None;
        }
        let (picked, rect) = menu(context, pivot, position, theme, fps, saving);
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

pub fn canvas_area(screen: Rect, edit_enabled: bool) -> Rect {
    let mut area = screen;
    if is_compact(screen) {
        area.min.y += MARGIN + FAB_SIZE;
        area.max.y -= TOOL_BAR_MARGIN + TOOL_BAR_HEIGHT;
    } else {
        if edit_enabled {
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

fn fab(ui: &mut Ui, chrome: &Chrome, icon: Icon, label: &str, on: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(FAB_SIZE), Sense::click());
    let fill = if on {
        chrome.accent
    } else if response.hovered() {
        chrome.hovered
    } else {
        chrome.panel
    };
    chrome.paint_floating(ui.painter(), rect, PANEL_RADIUS, fill);
    icons::paint_icon(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), Vec2::splat(ICON_SIZE)),
        if on { chrome.on_accent } else { chrome.text },
    );
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, on, label));
    response.on_hover_text(label)
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

fn corner_buttons(context: &Context, theme: Theme, edit_enabled: &mut bool) -> bool {
    let chrome = Chrome::of(theme);
    let mut examples_clicked = false;
    egui::Area::new("corner_buttons".into())
        .anchor(Align2::RIGHT_BOTTOM, [-MARGIN, -MARGIN])
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = FAB_GAP;
                examples_clicked = fab(ui, chrome, Icon::Grid, "Examples", false).clicked();
                if fab(ui, chrome, Icon::Pencil, "Edit", *edit_enabled).clicked() {
                    *edit_enabled = !*edit_enabled;
                }
            });
        });
    examples_clicked
}

fn menu(
    context: &Context,
    pivot: Align2,
    position: Pos2,
    theme: &mut Theme,
    fps: f64,
    saving: Saving,
) -> (Option<MenuChoice>, Rect) {
    let chrome = Chrome::of(*theme);
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
                        theme,
                        &[(Theme::Dark, "Dark"), (Theme::Light, "Light")],
                    )
                });
                labelled_row(ui, chrome, "Frame rate", |ui| {
                    ui.label(
                        RichText::new(format!("{fps:.0} fps"))
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
