mod icons;
mod inspector;
mod rail;
mod style;
#[cfg(test)]
mod tests;
mod widgets;

use std::path::PathBuf;

use egui::{
    pos2, vec2, Align, Align2, Context, Id, Layout, Order, Pos2, Rect, Response, RichText, Sense,
    Ui, Vec2, WidgetInfo, WidgetType,
};
use hecs::World;

use crate::editor::Editor;
use crate::rendering::Theme;
use icons::Icon;
use inspector::Inspector;
use rail::{Rail, POPOVER_GAP};
use style::{Chrome, MARGIN, PANEL_RADIUS};
use widgets::{icon_row, segmented, separator};

const FAB_SIZE: f32 = 46.0;
const FAB_GAP: f32 = 8.0;
const ICON_SIZE: f32 = 22.0;
const MENU_WIDTH: f32 = 232.0;
const MENU_PADDING: i8 = 6;
const MENU_ROW_HEIGHT: f32 = 34.0;

pub enum MenuChoice {
    Save(PathBuf),
    Open(PathBuf),
    Examples,
}

pub struct Controls<'a> {
    pub edit_enabled: &'a mut bool,
    pub editor: &'a mut Editor,
    pub world: &'a mut World,
    pub theme: &'a mut Theme,
    pub time: f64,
    pub fps: f64,
}

#[derive(Default)]
pub struct Gui {
    rail: Rail,
    inspector: Inspector,
    menu_open: bool,
}

impl Gui {
    pub fn show(&mut self, context: &Context, controls: Controls) -> Option<MenuChoice> {
        let theme = *controls.theme;
        let mut choice = None;
        if corner_buttons(context, theme, controls.edit_enabled) {
            choice = Some(MenuChoice::Examples);
        }
        let (menu_button, menu_position) = if *controls.edit_enabled {
            let rail = self
                .rail
                .show(context, theme, self.menu_open, controls.editor);
            self.inspector.show(
                context,
                theme,
                controls.world,
                controls.editor,
                controls.time,
            );
            let position = pos2(
                rail.rect.right() + POPOVER_GAP,
                rail.menu.rect.top() - MENU_PADDING as f32,
            );
            (rail.menu, position)
        } else {
            self.rail.close();
            let button = menu_button(context, theme, self.menu_open);
            let position = pos2(button.rect.left(), button.rect.bottom() + FAB_GAP);
            (button, position)
        };
        if menu_button.clicked() {
            self.menu_open = !self.menu_open;
        }
        if self.menu_open {
            let (picked, rect) = menu(context, menu_position, controls.theme, controls.fps);
            let pressed_elsewhere = context.input(|input| {
                input.pointer.any_pressed()
                    && input.pointer.interact_pos().is_some_and(|position| {
                        !rect.contains(position) && !menu_button.rect.contains(position)
                    })
            });
            if picked.is_some() || pressed_elsewhere {
                self.menu_open = false;
            }
            choice = choice.or(picked.flatten());
        }
        choice
    }
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

fn menu_button(context: &Context, theme: Theme, open: bool) -> Response {
    egui::Area::new("menu_button".into())
        .anchor(Align2::LEFT_TOP, [MARGIN, MARGIN])
        .show(context, |ui| {
            fab(ui, Chrome::of(theme), Icon::Menu, "Menu", open)
        })
        .inner
}

fn menu(
    context: &Context,
    position: Pos2,
    theme: &mut Theme,
    fps: f64,
) -> (Option<Option<MenuChoice>>, Rect) {
    let chrome = Chrome::of(*theme);
    let mut picked = None;
    let rect = egui::Area::new(Id::new("menu"))
        .order(Order::Foreground)
        .fixed_pos(position)
        .show(context, |ui| {
            chrome.panel_frame(MENU_PADDING).show(ui, |ui| {
                ui.set_width(MENU_WIDTH);
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if icon_row(ui, chrome, Icon::Folder, "Open…").clicked() {
                        picked = Some(rfd::FileDialog::new().pick_file().map(MenuChoice::Open));
                    }
                    if icon_row(ui, chrome, Icon::Save, "Save…").clicked() {
                        picked = Some(rfd::FileDialog::new().save_file().map(MenuChoice::Save));
                    }
                }
                if icon_row(ui, chrome, Icon::Grid, "Examples").clicked() {
                    picked = Some(Some(MenuChoice::Examples));
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

pub fn settings_window(context: &Context, iterations: &mut u32) {
    egui::Window::new("Settings").show(context, |ui| {
        ui.label("Simulation speed");
        ui.add(egui::Slider::new(iterations, 1..=20));
    });
}
