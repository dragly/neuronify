use egui::{Color32, CornerRadius, Painter, Rect, Shadow, Stroke, StrokeKind};

use crate::rendering::Theme;

pub const MARGIN: f32 = 12.0;
pub const PANEL_RADIUS: u8 = 14;

pub struct Chrome {
    pub panel: Color32,
    pub hovered: Color32,
    pub border: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub shadow: Color32,
}

const DARK: Chrome = Chrome {
    panel: Color32::from_rgb(30, 30, 46),
    hovered: Color32::from_rgb(69, 71, 90),
    border: Color32::from_rgb(69, 71, 90),
    text: Color32::from_rgb(205, 214, 244),
    accent: Color32::from_rgb(137, 180, 250),
    on_accent: Color32::from_rgb(17, 17, 27),
    shadow: Color32::from_black_alpha(96),
};

const LIGHT: Chrome = Chrome {
    panel: Color32::from_rgb(255, 255, 255),
    hovered: Color32::from_rgb(228, 232, 240),
    border: Color32::from_rgb(208, 213, 224),
    text: Color32::from_rgb(76, 79, 105),
    accent: Color32::from_rgb(30, 102, 245),
    on_accent: Color32::from_rgb(255, 255, 255),
    shadow: Color32::from_black_alpha(40),
};

impl Chrome {
    pub fn of(theme: Theme) -> &'static Chrome {
        match theme {
            Theme::Dark => &DARK,
            Theme::Light => &LIGHT,
        }
    }

    fn panel_shadow(&self) -> Shadow {
        Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: self.shadow,
        }
    }

    pub fn paint_floating(&self, painter: &Painter, rect: Rect, radius: u8, fill: Color32) {
        painter.add(
            self.panel_shadow()
                .as_shape(rect, CornerRadius::same(radius)),
        );
        painter.rect(
            rect,
            radius,
            fill,
            Stroke::new(1.0_f32, self.border),
            StrokeKind::Inside,
        );
    }
}
