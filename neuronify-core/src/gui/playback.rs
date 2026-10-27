use egui::{
    pos2, vec2, Align2, Context, FontId, Id, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo,
    WidgetType,
};

use crate::rendering::Theme;

use super::icons::{paint_icon, Icon};
use super::style::Chrome;

pub const PLAY: &str = "playback_play";
pub const SLOWER: &str = "playback_slower";
pub const FASTER: &str = "playback_faster";
pub const BAR_HEIGHT: f32 = 44.0;
pub const MAX_SPEED: u32 = 20;
const PADDING: i8 = 5;
const BUTTON: f32 = 34.0;
const ICON: f32 = 18.0;
const SPEED_WIDTH: f32 = 36.0;
const TIME_WIDTH: f32 = 78.0;

pub fn bar(
    context: &Context,
    theme: Theme,
    anchor: Align2,
    offset: Vec2,
    running: &mut bool,
    iterations: &mut u32,
    time: f64,
) {
    let chrome = Chrome::of(theme);
    egui::Area::new(Id::new("playback"))
        .anchor(anchor, offset)
        .show(context, |ui| {
            chrome
                .panel_frame(PADDING)
                .corner_radius(BAR_HEIGHT / 2.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        let (icon, label) = if *running {
                            (Icon::Pause, "Pause")
                        } else {
                            (Icon::Play, "Play")
                        };
                        if button(ui, chrome, icon, label, PLAY, true).clicked() {
                            *running = !*running;
                        }
                        divider(ui, chrome);
                        if button(ui, chrome, Icon::Minus, "Slower", SLOWER, false).clicked() {
                            *iterations = iterations.saturating_sub(1).max(1);
                        }
                        text(
                            ui,
                            SPEED_WIDTH,
                            format!("×{iterations}"),
                            FontId::monospace(13.0),
                            chrome.text,
                        )
                        .on_hover_text("Simulation steps per frame");
                        if button(ui, chrome, Icon::Plus, "Faster", FASTER, false).clicked() {
                            *iterations = (*iterations + 1).min(MAX_SPEED);
                        }
                        divider(ui, chrome);
                        text(
                            ui,
                            TIME_WIDTH,
                            format!("{time:.3} s"),
                            FontId::monospace(12.5),
                            chrome.subtext,
                        )
                        .on_hover_text("Simulated time");
                    });
                });
        });
}

fn button(ui: &mut Ui, chrome: &Chrome, icon: Icon, label: &str, id: &str, main: bool) -> Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(BUTTON), Sense::hover());
    let response = ui.interact(rect, Id::new(id), Sense::click());
    let painter = ui.painter();
    if main {
        painter.circle_filled(rect.center(), BUTTON / 2.0, chrome.accent);
    } else if response.hovered() {
        painter.circle_filled(rect.center(), BUTTON / 2.0, chrome.raised);
    }
    paint_icon(
        painter,
        icon,
        Rect::from_center_size(rect.center(), Vec2::splat(ICON)),
        if main { chrome.on_accent } else { chrome.text },
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response.on_hover_text(label)
}

fn divider(ui: &mut Ui, chrome: &Chrome) {
    let (rect, _) = ui.allocate_exact_size(vec2(13.0, BUTTON), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range().shrink(6.0),
        Stroke::new(1.0_f32, chrome.border),
    );
}

fn text(ui: &mut Ui, width: f32, text: String, font: FontId, color: egui::Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, BUTTON), Sense::hover());
    ui.painter().text(
        pos2(rect.center().x, rect.center().y),
        Align2::CENTER_CENTER,
        text,
        font,
        color,
    );
    response
}
