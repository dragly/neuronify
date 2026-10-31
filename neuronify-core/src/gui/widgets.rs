use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Rect, Response, Sense, Stroke, Ui, Vec2,
    WidgetInfo, WidgetType,
};

use super::icons::{paint_icon, Icon};
use super::style::Chrome;

pub const ROW_RADIUS: u8 = 8;
const ROW_HEIGHT: f32 = 34.0;
pub const ROW_INSET: f32 = 8.0;
const ROW_ICON: f32 = 18.0;
const SWITCH_SIZE: Vec2 = vec2(34.0, 20.0);
const SWITCH_KNOB: f32 = 7.0;

pub fn separator(ui: &mut Ui, chrome: &Chrome) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
    ui.painter().hline(
        rect.x_range().shrink(6.0),
        rect.center().y,
        Stroke::new(1.0_f32, chrome.border),
    );
}

pub fn heading(ui: &mut Ui, chrome: &Chrome, text: &str, inset: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    ui.painter().text(
        pos2(rect.left() + inset, rect.center().y),
        Align2::LEFT_CENTER,
        text.to_uppercase(),
        FontId::proportional(11.0),
        chrome.subtext,
    );
}

pub fn row(ui: &mut Ui, chrome: &Chrome, label: &str, sense: Sense) -> (Rect, Response) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), sense);
    if response.hovered() && sense.senses_click() {
        ui.painter().rect_filled(rect, ROW_RADIUS, chrome.raised);
    }
    let text_left = if sense.senses_click() {
        rect.left() + 2.0 * ROW_INSET + ROW_ICON
    } else {
        rect.left() + ROW_INSET
    };
    ui.painter().text(
        pos2(text_left, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.5),
        chrome.text,
    );
    (rect, response)
}

pub fn icon_row(ui: &mut Ui, chrome: &Chrome, icon: Icon, label: &str) -> Response {
    let (rect, response) = row(ui, chrome, label, Sense::click());
    paint_icon(
        ui.painter(),
        icon,
        Rect::from_center_size(
            pos2(rect.left() + ROW_INSET + ROW_ICON / 2.0, rect.center().y),
            Vec2::splat(ROW_ICON),
        ),
        chrome.text,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response
}

pub fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    chrome: &Chrome,
    value: &mut T,
    options: &[(T, &str)],
) {
    let font = FontId::proportional(12.5);
    let widths: Vec<f32> = options
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE)
                .size()
                .x
                + 18.0
        })
        .collect();
    let size = vec2(widths.iter().sum::<f32>() + 4.0, 28.0);
    let (track, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(track, ROW_RADIUS, chrome.raised);
    let mut left = track.left() + 2.0;
    for ((option, label), width) in options.iter().zip(widths) {
        let rect = Rect::from_min_size(pos2(left, track.top() + 2.0), vec2(width, size.y - 4.0));
        left += width;
        let response = ui.interact(rect, ui.id().with(label), Sense::click());
        let chosen = *value == *option;
        if chosen {
            ui.painter().rect_filled(rect, ROW_RADIUS - 2, chrome.panel);
        }
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            *label,
            font.clone(),
            if chosen || response.hovered() {
                chrome.text
            } else {
                chrome.subtext
            },
        );
        if response.clicked() {
            *value = *option;
        }
        response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, chosen, *label));
    }
}

pub fn paint_flyout_marker(painter: &Painter, tip: egui::Pos2, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![tip + vec2(-4.0, -4.0), tip, tip + vec2(-4.0, 4.0)],
        color,
        Stroke::NONE,
    ));
}

pub fn switch(ui: &mut Ui, chrome: &Chrome, label: &str, on: &mut bool) -> Response {
    let (rect, mut response) = row(ui, chrome, "", Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let painter = ui.painter();
    painter.text(
        pos2(rect.left() + ROW_INSET, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        chrome.text,
    );
    let track = Rect::from_center_size(
        pos2(
            rect.right() - ROW_INSET - SWITCH_SIZE.x / 2.0,
            rect.center().y,
        ),
        SWITCH_SIZE,
    );
    painter.rect_filled(
        track,
        SWITCH_SIZE.y / 2.0,
        if *on { chrome.accent } else { chrome.hovered },
    );
    let knob_offset = SWITCH_SIZE.x / 2.0 - SWITCH_SIZE.y / 2.0;
    painter.circle_filled(
        track.center() + vec2(if *on { knob_offset } else { -knob_offset }, 0.0),
        SWITCH_KNOB,
        chrome.panel,
    );
    let on = *on;
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, on, label));
    response
}
