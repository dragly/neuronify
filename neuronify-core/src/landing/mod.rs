mod preview;
mod raster;
mod top_down;

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use egui::{
    pos2, vec2, Align, Color32, CursorIcon, FontId, Galley, Key, Layout, Painter, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui, WidgetInfo, WidgetType,
};
use hecs::World;

use crate::examples::EXAMPLE_CATEGORIES;
use crate::rendering::Theme;
use preview::Preview;

struct Colors {
    background: Color32,
    surface: Color32,
    surface_hovered: Color32,
    accent: Color32,
    text: Color32,
    subtext: Color32,
}

const DARK: Colors = Colors {
    background: Color32::from_rgb(30, 30, 46),
    surface: Color32::from_rgb(49, 50, 68),
    surface_hovered: Color32::from_rgb(69, 71, 90),
    accent: Color32::from_rgb(137, 180, 250),
    text: Color32::from_rgb(205, 214, 244),
    subtext: Color32::from_rgb(166, 173, 200),
};

const LIGHT: Colors = Colors {
    background: Color32::from_rgb(236, 239, 245),
    surface: Color32::from_rgb(255, 255, 255),
    surface_hovered: Color32::from_rgb(246, 248, 253),
    accent: Color32::from_rgb(30, 102, 245),
    text: Color32::from_rgb(76, 79, 105),
    subtext: Color32::from_rgb(92, 95, 119),
};

impl Colors {
    fn of(theme: Theme) -> &'static Colors {
        match theme {
            Theme::Dark => &DARK,
            Theme::Light => &LIGHT,
        }
    }
}

const MAX_CONTENT_WIDTH: f32 = 1100.0;
const PAGE_MARGIN: f32 = 16.0;
const MIN_CARD_WIDTH: f32 = 240.0;
const CARD_GAP: f32 = 16.0;
const CARD_PADDING: f32 = 10.0;
const CARD_RADIUS: u8 = 10;
const TITLE_GAP: f32 = 4.0;
const THUMBNAIL_ASPECT: f32 = 9.0 / 16.0;
const THUMBNAIL_RADIUS: u8 = 6;
const STILL_DRAWING_MICROSECONDS_PER_FRAME: i64 = 2000;

pub enum LandingChoice {
    Example { world: Box<World>, time: f64 },
    EmptyCanvas,
    BackToSimulation,
}

#[derive(Clone, Copy)]
enum Card {
    EmptyCanvas,
    Preview(usize),
}

enum Picked {
    Card(usize, Card),
    BackToSimulation,
}

pub struct LandingScreen {
    previews_by_category: Vec<Vec<Preview>>,
    can_go_back: bool,
    theme: Theme,
    still_drawing_deadline: Option<DateTime<Utc>>,
}

impl LandingScreen {
    pub fn new(can_go_back: bool) -> LandingScreen {
        let previews_by_category = EXAMPLE_CATEGORIES
            .iter()
            .map(|category| category.examples.iter().filter_map(Preview::new).collect())
            .collect();
        LandingScreen {
            previews_by_category,
            can_go_back,
            theme: Theme::default(),
            still_drawing_deadline: None,
        }
    }

    pub fn update(&mut self, seconds: f64, physics_steps: u32) {
        for preview in self.previews_by_category.iter_mut().flatten() {
            preview.step(seconds, physics_steps);
        }
    }

    pub fn show(&mut self, context: &egui::Context, theme: Theme) -> Option<LandingChoice> {
        let mut picked = None;
        self.theme = theme;
        self.still_drawing_deadline =
            Some(Utc::now() + Duration::microseconds(STILL_DRAWING_MICROSECONDS_PER_FRAME));
        #[allow(deprecated)]
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Colors::of(theme).background))
            .show(context, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width =
                            (ui.available_width() - 2.0 * PAGE_MARGIN).min(MAX_CONTENT_WIDTH);
                        ui.horizontal_top(|ui| {
                            ui.add_space((ui.available_width() - width) / 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                picked = self.contents(ui, width);
                            });
                        });
                    });
            });
        if self.can_go_back && context.input(|input| input.key_pressed(Key::Escape)) {
            picked = Some(Picked::BackToSimulation);
        }
        match picked? {
            Picked::Card(_, Card::EmptyCanvas) => Some(LandingChoice::EmptyCanvas),
            Picked::Card(category, Card::Preview(index)) => {
                let preview = self.previews_by_category[category].swap_remove(index);
                Some(LandingChoice::Example {
                    world: Box::new(preview.world),
                    time: preview.time,
                })
            }
            Picked::BackToSimulation => Some(LandingChoice::BackToSimulation),
        }
    }

    fn contents(&mut self, ui: &mut Ui, width: f32) -> Option<Picked> {
        let colors = Colors::of(self.theme);
        let mut picked = None;
        ui.add_space(40.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Neuronify").size(34.0).color(colors.text));
            if self.can_go_back {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let button = egui::Button::new(
                        RichText::new("Back to simulation")
                            .size(15.0)
                            .color(colors.text),
                    )
                    .fill(colors.surface)
                    .corner_radius(CARD_RADIUS)
                    .min_size(vec2(0.0, 36.0));
                    if ui.add(button).clicked() {
                        picked = Some(Picked::BackToSimulation);
                    }
                });
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Simulations of neurons and small circuits. \
                 Open an example below, or start from an empty canvas.",
            )
            .size(16.0)
            .color(colors.subtext),
        );

        for (category_index, category) in EXAMPLE_CATEGORIES.iter().enumerate() {
            ui.add_space(32.0);
            ui.label(RichText::new(category.name).size(20.0).color(colors.text));
            ui.add_space(12.0);
            let mut cards = Vec::new();
            if category_index == 0 {
                cards.push(Card::EmptyCanvas);
            }
            cards.extend((0..self.previews_by_category[category_index].len()).map(Card::Preview));
            if let Some(card) = self.card_grid(ui, width, category_index, &cards) {
                picked = Some(Picked::Card(category_index, card));
            }
        }
        ui.add_space(40.0);
        picked
    }

    fn card_texts(&self, ui: &Ui, category: usize, card: Card, width: f32) -> [Arc<Galley>; 2] {
        let (name, description) = match card {
            Card::EmptyCanvas => ("Empty canvas", "Build your own circuit from scratch."),
            Card::Preview(index) => {
                let example = self.previews_by_category[category][index].example;
                (example.name, example.description)
            }
        };
        let colors = Colors::of(self.theme);
        let painter = ui.painter();
        [
            painter.layout(
                name.to_owned(),
                FontId::proportional(16.0),
                colors.text,
                width,
            ),
            painter.layout(
                description.to_owned(),
                FontId::proportional(13.5),
                colors.subtext,
                width,
            ),
        ]
    }

    fn card_grid(
        &mut self,
        ui: &mut Ui,
        width: f32,
        category: usize,
        cards: &[Card],
    ) -> Option<Card> {
        let columns = (((width + CARD_GAP) / (MIN_CARD_WIDTH + CARD_GAP)) as usize).max(1);
        let card_width = (width - CARD_GAP * (columns - 1) as f32) / columns as f32;
        let text_width = card_width - 2.0 * CARD_PADDING;
        let thumbnail_size = vec2(text_width, (text_width * THUMBNAIL_ASPECT).round());
        let theme = self.theme;
        let colors = Colors::of(theme);
        let mut clicked = None;
        for row in cards.chunks(columns) {
            let texts: Vec<[Arc<Galley>; 2]> = row
                .iter()
                .map(|&card| self.card_texts(ui, category, card, text_width))
                .collect();
            let tallest_text_in_row = texts
                .iter()
                .map(|[name, description]| name.size().y + TITLE_GAP + description.size().y)
                .fold(0.0, f32::max);
            let card_size = vec2(
                card_width,
                3.0 * CARD_PADDING + thumbnail_size.y + tallest_text_in_row,
            );
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                for (&card, [name, description]) in row.iter().zip(texts) {
                    let (rect, response) = ui.allocate_exact_size(card_size, Sense::click());
                    let on_screen = ui.is_rect_visible(rect);
                    if let Card::Preview(index) = card {
                        self.previews_by_category[category][index].on_screen = on_screen;
                    }
                    if !on_screen {
                        continue;
                    }
                    let highlighted = response.hovered() || response.has_focus();
                    let painter = ui.painter();
                    painter.rect(
                        rect,
                        CARD_RADIUS,
                        if highlighted {
                            colors.surface_hovered
                        } else {
                            colors.surface
                        },
                        if highlighted {
                            Stroke::new(1.5_f32, colors.accent)
                        } else {
                            Stroke::NONE
                        },
                        StrokeKind::Inside,
                    );
                    let thumbnail = Rect::from_min_size(
                        rect.min + vec2(CARD_PADDING, CARD_PADDING),
                        thumbnail_size,
                    );
                    painter.rect_filled(thumbnail, THUMBNAIL_RADIUS, colors.background);
                    let thumbnail_painter = ui.painter_at(thumbnail);
                    match card {
                        Card::EmptyCanvas => {
                            paint_plus(&thumbnail_painter, thumbnail, colors.subtext)
                        }
                        Card::Preview(index) => self.previews_by_category[category][index].paint(
                            &thumbnail_painter,
                            thumbnail,
                            theme,
                            &mut self.still_drawing_deadline,
                        ),
                    }
                    let name_position = pos2(
                        rect.left() + CARD_PADDING,
                        thumbnail.bottom() + CARD_PADDING,
                    );
                    let description_position = name_position + vec2(0.0, name.size().y + TITLE_GAP);
                    let label = name.text().to_owned();
                    painter.galley(name_position, name, colors.text);
                    painter.galley(description_position, description, colors.subtext);
                    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &label));
                    if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        clicked = Some(card);
                    }
                }
            });
            ui.add_space(CARD_GAP);
        }
        clicked
    }
}

fn paint_plus(painter: &Painter, area: Rect, color: Color32) {
    let center = area.center();
    let arm = area.height() * 0.15;
    let stroke = Stroke::new(2.0_f32, color);
    painter.line_segment([center - vec2(arm, 0.0), center + vec2(arm, 0.0)], stroke);
    painter.line_segment([center - vec2(0.0, arm), center + vec2(0.0, arm)], stroke);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::landing::preview::{PreviewMode, PREVIEW_FADE_IN_SECONDS};
    use egui::{Event, PointerButton, Pos2, RawInput, Shape, Vec2};

    fn run(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        run_sized(context, landing, events, vec2(1280.0, 800.0))
    }

    fn run_sized(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
        screen: Vec2,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        run_at(context, landing, events, screen, 0.0, Theme::Dark)
    }

    fn run_at(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
        screen: Vec2,
        time: f64,
        theme: Theme,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut choice = None;
        let output = context.run_ui(input, |ui| choice = landing.show(ui.ctx(), theme));
        (output, choice)
    }

    fn text_position(output: &egui::FullOutput, text: &str) -> Option<Pos2> {
        output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => Some(shape.pos),
                _ => None,
            })
    }

    #[test]
    fn clicking_a_card_opens_its_example() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let (output, _) = run(&context, &mut landing, vec![]);
        let position = text_position(&output, "Tutorial 1: Intro").unwrap() + vec2(4.0, 4.0);

        let click = |pressed| Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        run(
            &context,
            &mut landing,
            vec![Event::PointerMoved(position), click(true)],
        );
        let (_, choice) = run(&context, &mut landing, vec![click(false)]);

        let Some(LandingChoice::Example { world, .. }) = choice else {
            panic!("clicking the card did not open an example");
        };
        assert_eq!(world.query::<&LeakyNeuron>().iter().count(), 1);
    }

    #[test]
    fn escape_goes_back_only_when_there_is_something_to_go_back_to() {
        let escape = || {
            vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }]
        };
        let context = egui::Context::default();
        let (_, choice) = run(&context, &mut LandingScreen::new(false), escape());
        assert!(choice.is_none());
        let (_, choice) = run(&context, &mut LandingScreen::new(true), escape());
        assert!(matches!(choice, Some(LandingChoice::BackToSimulation)));
    }

    #[test]
    fn previews_out_of_view_do_not_run() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let window_with_room_for_one_row = vec2(1280.0, 400.0);
        run_sized(&context, &mut landing, vec![], window_with_room_for_one_row);
        landing.update(1.0 / 60.0, 4);
        let first = &landing.previews_by_category[0][0];
        let last = landing.previews_by_category.last().unwrap().last().unwrap();
        assert!(first.on_screen && first.time > 0.0);
        assert!(!last.on_screen && last.time == 0.0);
    }

    #[test]
    fn follows_the_theme() {
        let screen = vec2(1280.0, 800.0);
        for theme in [Theme::Dark, Theme::Light] {
            let context = egui::Context::default();
            let mut landing = LandingScreen::new(false);
            let (output, _) = run_at(&context, &mut landing, vec![], screen, 0.0, theme);
            let background = Colors::of(theme).background;
            assert!(output.shapes.iter().any(|clipped| matches!(
                &clipped.shape,
                Shape::Rect(rect) if rect.fill == background
            )));
        }
    }

    #[test]
    fn still_previews_are_drawn_again_when_the_theme_changes() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        landing.previews_by_category[0][0].mode = PreviewMode::Still;
        for theme in [Theme::Dark, Theme::Light] {
            run_at(
                &context,
                &mut landing,
                vec![],
                vec2(1280.0, 800.0),
                0.0,
                theme,
            );
            let preview = &landing.previews_by_category[0][0];
            assert_eq!(preview.still_drawing.as_ref().unwrap().theme, theme);
        }
    }

    #[test]
    fn live_previews_are_hidden_while_warming_up_and_then_fade_in() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let screen = vec2(1280.0, 800.0);
        let neuron_opacities = |output: &egui::FullOutput| -> Vec<u8> {
            output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    Shape::Circle(circle) => Some(circle.fill.a()),
                    _ => None,
                })
                .collect()
        };
        let mut frame = 0;
        let mut next_frame = |landing: &mut LandingScreen| {
            landing.update(1.0 / 60.0, 4);
            frame += 1;
            run_at(
                &context,
                landing,
                vec![],
                screen,
                frame as f64 / 60.0,
                Theme::Dark,
            )
            .0
        };

        let output = next_frame(&mut landing);
        assert!(neuron_opacities(&output).is_empty());
        while !landing.previews_by_category[0][0].warmed_up() {
            let output = next_frame(&mut landing);
            assert!(neuron_opacities(&output).is_empty());
        }
        let fade_frames = (PREVIEW_FADE_IN_SECONDS * 60.0) as usize;
        let mut output = next_frame(&mut landing);
        for _ in 0..fade_frames / 2 {
            output = next_frame(&mut landing);
        }
        let halfway_through_fade = neuron_opacities(&output);
        assert!(
            halfway_through_fade.iter().any(|&a| 0 < a && a < 255),
            "{halfway_through_fade:?}"
        );
        for _ in 0..fade_frames {
            output = next_frame(&mut landing);
        }
        let shown = neuron_opacities(&output);
        assert!(!shown.is_empty() && shown.iter().all(|&a| a == 255));
    }
}
