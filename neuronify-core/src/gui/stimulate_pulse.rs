use egui::{Context, CornerRadius, Id, LayerId, Order, Rect, Stroke, StrokeKind};

use super::style::Chrome;
use crate::components::Seconds;
use crate::rendering::Theme;

const PULSE: f64 = 1.3;
const PULSES: f64 = 3.0;
const REST: f64 = 4.0;
const GROWTH: f32 = 12.0;
const WIDTH: f32 = 2.5;
const GLOW: f32 = 0.18;
const RADIUS: u8 = 10;

pub fn paint(context: &Context, theme: Theme, button: Rect, age: Seconds) {
    context.request_repaint();
    let within = age.0.rem_euclid(PULSES * PULSE + REST);
    if within >= PULSES * PULSE {
        return;
    }
    let progress = ((within % PULSE) / PULSE) as f32;
    let chrome = Chrome::of(theme);
    let painter =
        context.layer_painter(LayerId::new(Order::Foreground, Id::new("stimulate_pulse")));
    let growth = GROWTH * progress;
    painter.rect_filled(
        button,
        CornerRadius::same(RADIUS),
        chrome.accent.gamma_multiply(GLOW * (1.0 - progress)),
    );
    painter.rect_stroke(
        button.expand(growth),
        CornerRadius::same(RADIUS + growth as u8),
        Stroke::new(
            WIDTH,
            chrome.accent.gamma_multiply((1.0 - progress).powi(2)),
        ),
        StrokeKind::Outside,
    );
}
