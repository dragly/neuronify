use egui::{vec2, Context, Id, Rect, Response, Sense, Ui, WidgetInfo, WidgetType};

use super::style::Chrome;

pub const HANDLE_HEIGHT: f32 = 26.0;
const SETTLE_TIME: f32 = 0.2;
const THROW_TIME: f32 = 0.15;

#[derive(Default)]
pub struct Sheet {
    detent: usize,
    dragged: Option<f32>,
    shown: bool,
}

impl Sheet {
    pub fn height(&mut self, context: &Context, id: Id, detents: &[f32]) -> f32 {
        let height = self.dragged.unwrap_or(detents[self.detent]);
        if !self.shown {
            self.shown = true;
            context.request_discard("a sheet appeared");
        } else if self.dragged.is_none() {
            return context.animate_value_with_time(id, height, SETTLE_TIME);
        }
        context.animate_value_with_time(id, height, 0.0);
        height
    }

    pub fn handle(&mut self, ui: &mut Ui, chrome: &Chrome, height: f32, detents: &[f32]) -> bool {
        let top = detents.len() - 1;
        let label = if top == 0 {
            "Close"
        } else if self.detent == top {
            "Shrink"
        } else {
            "Expand"
        };
        let response = handle(ui, chrome, label);
        if response.dragged() {
            let dragged = self.dragged.unwrap_or(height) - response.drag_delta().y;
            self.dragged = Some(dragged.clamp(0.0, detents[top]));
        }
        if response.drag_stopped() {
            let released = self.dragged.take().unwrap_or(height);
            let thrown = released - ui.input(|input| input.pointer.velocity().y) * THROW_TIME;
            if thrown < detents[0] / 2.0 {
                *self = Sheet::default();
                return true;
            }
            self.detent = (0..detents.len())
                .min_by(|&a, &b| {
                    (detents[a] - thrown)
                        .abs()
                        .total_cmp(&(detents[b] - thrown).abs())
                })
                .unwrap_or_default();
        }
        if response.clicked() {
            if top == 0 {
                return true;
            }
            self.detent = (self.detent + 1) % detents.len();
        }
        false
    }
}

pub fn layout(height: f32, detents: &[f32]) -> (f32, f32) {
    (height.max(detents[0]), (detents[0] - height).max(0.0))
}

fn handle(ui: &mut Ui, chrome: &Chrome, label: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), HANDLE_HEIGHT),
        Sense::click_and_drag(),
    );
    ui.painter().rect_filled(
        Rect::from_center_size(rect.center(), vec2(40.0, 4.0)),
        2.0,
        if response.hovered() || response.dragged() {
            chrome.subtext
        } else {
            chrome.border
        },
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response
}
