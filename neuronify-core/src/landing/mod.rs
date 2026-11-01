mod dialogs;
mod preview;
mod raster;
mod top_down;

use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use egui::text::LayoutJob;
use egui::{
    pos2, vec2, Align, Align2, Color32, CursorIcon, FontId, Galley, Key, Layout, Painter, Rect,
    Response, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, WidgetInfo, WidgetType,
};
use hecs::World;

use crate::constants::DEFAULT_SPEED;
use crate::examples::{Example, EXAMPLE_CATEGORIES};
use crate::gui::icons::{paint_icon, Icon};
use crate::library::{Library, SavedSimulation};
use crate::rendering::Theme;
use dialogs::Dialogs;
use preview::Preview;

struct Colors {
    background: Color32,
    surface: Color32,
    surface_hovered: Color32,
    accent: Color32,
    danger: Color32,
    text: Color32,
    subtext: Color32,
}

const DARK: Colors = Colors {
    background: Color32::from_rgb(30, 30, 46),
    surface: Color32::from_rgb(49, 50, 68),
    surface_hovered: Color32::from_rgb(69, 71, 90),
    accent: Color32::from_rgb(137, 180, 250),
    danger: Color32::from_rgb(243, 139, 168),
    text: Color32::from_rgb(205, 214, 244),
    subtext: Color32::from_rgb(166, 173, 200),
};

const LIGHT: Colors = Colors {
    background: Color32::from_rgb(236, 239, 245),
    surface: Color32::from_rgb(255, 255, 255),
    surface_hovered: Color32::from_rgb(246, 248, 253),
    accent: Color32::from_rgb(30, 102, 245),
    danger: Color32::from_rgb(210, 15, 57),
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
const MIN_CARD_WIDTH: f32 = 200.0;
const CARDS_PER_SECTION: usize = 6;
const COMPACT_TEXT_ROWS: usize = 2;
const SECTION_HEADER_HEIGHT: f32 = 32.0;
const BACK_ICON_SIZE: f32 = 20.0;
const CARD_GAP: f32 = 16.0;
const CARD_PADDING: f32 = 10.0;
const CARD_RADIUS: u8 = 10;
const TITLE_GAP: f32 = 4.0;
const THUMBNAIL_ASPECT: f32 = 9.0 / 16.0;
const THUMBNAIL_RADIUS: u8 = 6;
const STILL_DRAWING_MICROSECONDS_PER_FRAME: i64 = 2000;
const YOUR_SIMULATIONS: &str = "Your simulations";

pub enum LandingChoice {
    NewSimulation,
    Saved {
        id: String,
        world: Box<World>,
        time: f64,
        speed: f64,
    },
    Example {
        name: &'static str,
        world: Box<World>,
        time: f64,
    },
    BackToSimulation,
}

#[derive(Clone, Copy)]
enum Card {
    NewSimulation,
    Saved(usize),
    Example { category: usize, index: usize },
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Section {
    Yours,
    Examples(usize),
}

impl Section {
    fn all() -> impl Iterator<Item = Section> {
        std::iter::once(Section::Yours).chain((0..EXAMPLE_CATEGORIES.len()).map(Section::Examples))
    }

    fn title(self) -> &'static str {
        match self {
            Section::Yours => YOUR_SIMULATIONS,
            Section::Examples(category) => EXAMPLE_CATEGORIES[category].name,
        }
    }
}

enum Picked {
    Card(Card),
    Section(Section),
    Overview,
    BackToSimulation,
}

enum CardAction {
    Rename(usize),
    Delete(usize),
}

struct ExampleCard {
    example: &'static Example,
    preview: Preview,
}

struct SavedCard {
    id: String,
    name: String,
    modified: DateTime<Utc>,
    speed: f64,
    preview: Result<Preview, String>,
}

impl SavedCard {
    fn new(simulation: &SavedSimulation) -> SavedCard {
        let loaded = crate::serialization::load(&simulation.bytes);
        SavedCard {
            id: simulation.id.clone(),
            name: simulation.name.clone(),
            modified: simulation.modified,
            speed: loaded.as_ref().map_or(DEFAULT_SPEED, |saved| saved.speed),
            preview: loaded.map(|saved| Preview::new(saved.world, saved.time)),
        }
    }
}

pub struct LandingScreen {
    examples_by_category: Vec<Vec<ExampleCard>>,
    saved: Vec<SavedCard>,
    library_revision: Option<u64>,
    dialogs: Dialogs,
    section: Option<Section>,
    can_go_back: bool,
    theme: Theme,
    still_drawing_deadline: Option<DateTime<Utc>>,
}

impl LandingScreen {
    pub fn new(can_go_back: bool) -> LandingScreen {
        let examples_by_category = EXAMPLE_CATEGORIES
            .iter()
            .map(|category| {
                category
                    .examples
                    .iter()
                    .filter_map(|example| {
                        let world = example
                            .spawn()
                            .map_err(|e| log::error!("Could not load {}: {e}", example.name))
                            .ok()?;
                        Some(ExampleCard {
                            example,
                            preview: Preview::new(world, 0.0),
                        })
                    })
                    .collect()
            })
            .collect();
        LandingScreen {
            examples_by_category,
            saved: Vec::new(),
            library_revision: None,
            dialogs: Dialogs::default(),
            section: None,
            can_go_back,
            theme: Theme::default(),
            still_drawing_deadline: None,
        }
    }

    pub fn update(&mut self, seconds: f64, physics_steps: u32) {
        for preview in self.previews() {
            preview.step(seconds, physics_steps);
        }
    }

    fn previews(&mut self) -> impl Iterator<Item = &mut Preview> {
        let saved = self
            .saved
            .iter_mut()
            .filter_map(|card| card.preview.as_mut().ok());
        let examples = self
            .examples_by_category
            .iter_mut()
            .flatten()
            .map(|card| &mut card.preview);
        saved.chain(examples)
    }

    pub fn show(
        &mut self,
        context: &egui::Context,
        theme: Theme,
        library: &mut Library,
    ) -> Option<LandingChoice> {
        self.follow(library);
        for preview in self.previews() {
            preview.on_screen = false;
        }
        let mut picked = None;
        self.theme = theme;
        self.still_drawing_deadline =
            Some(Utc::now() + Duration::microseconds(STILL_DRAWING_MICROSECONDS_PER_FRAME));
        #[allow(deprecated)]
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Colors::of(theme).background))
            .show(context, |ui| {
                ScrollArea::vertical()
                    .id_salt(self.section)
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
        let dialog_was_open = self.dialogs.is_open();
        self.dialogs.show(context, Colors::of(theme), library);
        if !dialog_was_open && context.input(|input| input.key_pressed(Key::Escape)) {
            if self.section.is_some() {
                picked = Some(Picked::Overview);
            } else if self.can_go_back {
                picked = Some(Picked::BackToSimulation);
            }
        }
        match picked? {
            Picked::Section(section) => {
                self.section = Some(section);
                None
            }
            Picked::Overview => {
                self.section = None;
                None
            }
            Picked::Card(Card::NewSimulation) => Some(LandingChoice::NewSimulation),
            Picked::Card(Card::Saved(index)) => {
                if self.saved[index].preview.is_err() {
                    return None;
                }
                let card = self.saved.swap_remove(index);
                let preview = card.preview.ok()?;
                Some(LandingChoice::Saved {
                    id: card.id,
                    world: Box::new(preview.world),
                    time: preview.time,
                    speed: card.speed,
                })
            }
            Picked::Card(Card::Example { category, index }) => {
                let card = self.examples_by_category[category].swap_remove(index);
                Some(LandingChoice::Example {
                    name: card.example.name,
                    world: Box::new(card.preview.world),
                    time: card.preview.time,
                })
            }
            Picked::BackToSimulation => Some(LandingChoice::BackToSimulation),
        }
    }

    fn follow(&mut self, library: &Library) {
        if self.library_revision == Some(library.revision()) {
            return;
        }
        self.library_revision = Some(library.revision());
        let mut previous = std::mem::take(&mut self.saved);
        self.saved = library
            .simulations()
            .iter()
            .map(|simulation| {
                let unchanged = previous.iter().position(|card| {
                    card.id == simulation.id && card.modified == simulation.modified
                });
                match unchanged {
                    Some(index) => SavedCard {
                        name: simulation.name.clone(),
                        ..previous.swap_remove(index)
                    },
                    None => SavedCard::new(simulation),
                }
            })
            .collect();
    }

    fn contents(&mut self, ui: &mut Ui, width: f32) -> Option<Picked> {
        match self.section {
            Some(section) => self.section_page(ui, width, section),
            None => self.overview(ui, width),
        }
    }

    fn overview(&mut self, ui: &mut Ui, width: f32) -> Option<Picked> {
        let colors = Colors::of(self.theme);
        let mut picked = None;
        ui.add_space(if columns(width) == 2 { 24.0 } else { 40.0 });
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
            RichText::new("Simulations of neurons and small circuits.")
                .size(16.0)
                .color(colors.subtext),
        );

        for section in Section::all() {
            if let Some(section_picked) = self.section(ui, width, section) {
                picked = Some(section_picked);
            }
        }
        ui.add_space(40.0);
        picked
    }

    fn section(&mut self, ui: &mut Ui, width: f32, section: Section) -> Option<Picked> {
        let colors = Colors::of(self.theme);
        let cards = self.cards(section);
        let mut picked = None;
        ui.add_space(24.0);
        let (header, _) =
            ui.allocate_exact_size(vec2(width, SECTION_HEADER_HEIGHT), Sense::hover());
        ui.painter().text(
            header.left_center(),
            Align2::LEFT_CENTER,
            section.title(),
            FontId::proportional(20.0),
            colors.text,
        );
        if cards.len() > CARDS_PER_SECTION && see_all(ui, colors, header, section).clicked() {
            picked = Some(Picked::Section(section));
        }
        ui.add_space(8.0);
        let shown = &cards[..cards.len().min(CARDS_PER_SECTION)];
        if let Some(card) = self.card_grid(ui, width, shown) {
            picked = Some(Picked::Card(card));
        }
        picked
    }

    fn section_page(&mut self, ui: &mut Ui, width: f32, section: Section) -> Option<Picked> {
        let colors = Colors::of(self.theme);
        let mut picked = None;
        ui.add_space(16.0);
        if back(ui, colors).clicked() {
            picked = Some(Picked::Overview);
        }
        ui.add_space(4.0);
        ui.label(RichText::new(section.title()).size(34.0).color(colors.text));
        ui.add_space(24.0);
        let cards = self.cards(section);
        if let Some(card) = self.card_grid(ui, width, &cards) {
            picked = Some(Picked::Card(card));
        }
        ui.add_space(40.0);
        picked
    }

    fn cards(&self, section: Section) -> Vec<Card> {
        match section {
            Section::Yours => std::iter::once(Card::NewSimulation)
                .chain((0..self.saved.len()).map(Card::Saved))
                .collect(),
            Section::Examples(category) => (0..self.examples_by_category[category].len())
                .map(|index| Card::Example { category, index })
                .collect(),
        }
    }

    fn card_texts(&self, ui: &Ui, card: Card, width: f32, rows: usize) -> [Arc<Galley>; 2] {
        let (name, description, description_rows) = match card {
            Card::NewSimulation => (
                "New simulation".to_owned(),
                "Start from scratch.".to_owned(),
                rows,
            ),
            Card::Saved(index) => {
                let card = &self.saved[index];
                let description = match &card.preview {
                    Ok(_) => edited(
                        card.modified.with_timezone(&Local).naive_local(),
                        Local::now().date_naive(),
                    ),
                    Err(error) => error.clone(),
                };
                (card.name.clone(), description, 1)
            }
            Card::Example { category, index } => {
                let example = self.examples_by_category[category][index].example;
                (
                    example.name.to_owned(),
                    example.description.to_owned(),
                    rows,
                )
            }
        };
        let colors = Colors::of(self.theme);
        let layout = |text: String, size: f32, color: Color32, max_rows: usize| {
            let mut job = LayoutJob::simple(text, FontId::proportional(size), color, width);
            job.wrap.max_rows = max_rows;
            ui.painter().layout_job(job)
        };
        [
            layout(name, 16.0, colors.text, rows),
            layout(description, 13.5, colors.subtext, description_rows),
        ]
    }

    fn card_grid(&mut self, ui: &mut Ui, width: f32, cards: &[Card]) -> Option<Card> {
        let columns = columns(width);
        let text_rows = if columns == 2 {
            COMPACT_TEXT_ROWS
        } else {
            usize::MAX
        };
        let card_width = (width - CARD_GAP * (columns - 1) as f32) / columns as f32;
        let text_width = card_width - 2.0 * CARD_PADDING;
        let thumbnail_size = vec2(text_width, (text_width * THUMBNAIL_ASPECT).round());
        let theme = self.theme;
        let colors = Colors::of(theme);
        let mut clicked = None;
        let mut action = None;
        for row in cards.chunks(columns) {
            let texts: Vec<[Arc<Galley>; 2]> = row
                .iter()
                .map(|&card| self.card_texts(ui, card, text_width, text_rows))
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
                    let mut preview =
                        preview_mut(&mut self.saved, &mut self.examples_by_category, card);
                    if let Some(preview) = &mut preview {
                        preview.on_screen = on_screen;
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
                    match (card, preview) {
                        (Card::NewSimulation, _) => {
                            paint_plus(&thumbnail_painter, thumbnail, colors.subtext)
                        }
                        (_, Some(preview)) => preview.paint(
                            &thumbnail_painter,
                            thumbnail,
                            theme,
                            &mut self.still_drawing_deadline,
                        ),
                        (_, None) => {}
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
                    if let Card::Saved(index) = card {
                        response.context_menu(|ui| {
                            if menu_item(ui, colors.text, "Rename…").clicked() {
                                action = Some(CardAction::Rename(index));
                            }
                            if menu_item(ui, colors.danger, "Delete…").clicked() {
                                action = Some(CardAction::Delete(index));
                            }
                        });
                    }
                    if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        clicked = Some(card);
                    }
                }
            });
            ui.add_space(CARD_GAP);
        }
        match action {
            Some(CardAction::Rename(index)) => {
                let card = &self.saved[index];
                self.dialogs.rename(&card.id, &card.name);
            }
            Some(CardAction::Delete(index)) => {
                let card = &self.saved[index];
                self.dialogs.delete(&card.id, &card.name);
            }
            None => {}
        }
        clicked
    }
}

fn preview_mut<'a>(
    saved: &'a mut [SavedCard],
    examples_by_category: &'a mut [Vec<ExampleCard>],
    card: Card,
) -> Option<&'a mut Preview> {
    match card {
        Card::NewSimulation => None,
        Card::Saved(index) => saved[index].preview.as_mut().ok(),
        Card::Example { category, index } => {
            Some(&mut examples_by_category[category][index].preview)
        }
    }
}

fn menu_item(ui: &mut Ui, color: Color32, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(15.0).color(color))
            .frame(false)
            .min_size(vec2(160.0, 34.0)),
    )
}

fn columns(width: f32) -> usize {
    if width + CARD_GAP >= 3.0 * (MIN_CARD_WIDTH + CARD_GAP) {
        3
    } else {
        2
    }
}

fn see_all(ui: &mut Ui, colors: &Colors, header: Rect, section: Section) -> Response {
    let label = "See all";
    let text =
        ui.painter()
            .layout_no_wrap(label.to_owned(), FontId::proportional(15.0), colors.accent);
    let text_rect = Rect::from_min_size(
        pos2(
            header.right() - text.size().x,
            header.center().y - text.size().y / 2.0,
        ),
        text.size(),
    );
    let area = Rect::from_x_y_ranges(text_rect.x_range().expand(8.0), header.y_range());
    let response = ui.interact(area, ui.id().with(section), Sense::click());
    ui.painter().galley(text_rect.min, text, colors.accent);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn back(ui: &mut Ui, colors: &Colors) -> Response {
    let label = "Back";
    let text =
        ui.painter()
            .layout_no_wrap(label.to_owned(), FontId::proportional(16.0), colors.accent);
    let text_left = BACK_ICON_SIZE / 2.0;
    let (rect, response) = ui.allocate_exact_size(
        vec2(text_left + text.size().x, SECTION_HEADER_HEIGHT + 4.0),
        Sense::click(),
    );
    let icon = Rect::from_min_size(
        pos2(
            rect.left() - BACK_ICON_SIZE / 3.0,
            rect.center().y - BACK_ICON_SIZE / 2.0,
        ),
        vec2(BACK_ICON_SIZE, BACK_ICON_SIZE),
    );
    paint_icon(ui.painter(), Icon::Back, icon, colors.accent);
    ui.painter().galley(
        pos2(
            rect.left() + text_left,
            rect.center().y - text.size().y / 2.0,
        ),
        text,
        colors.accent,
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn edited(modified: NaiveDateTime, today: NaiveDate) -> String {
    let date = modified.date();
    let time = modified.format("%H:%M");
    if date == today {
        format!("Today at {time}")
    } else if today.pred_opt() == Some(date) {
        format!("Yesterday at {time}")
    } else if date.year() == today.year() {
        modified.format("%-d %B").to_string()
    } else {
        modified.format("%-d %B %Y").to_string()
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
        run_with(context, landing, &mut Library::default(), events)
    }

    fn run_with(
        context: &egui::Context,
        landing: &mut LandingScreen,
        library: &mut Library,
        events: Vec<Event>,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let window_with_room_for_every_section = vec2(1280.0, 2400.0);
        run_at(
            context,
            landing,
            library,
            events,
            window_with_room_for_every_section,
            0.0,
            Theme::Dark,
        )
    }

    fn run_sized(
        context: &egui::Context,
        landing: &mut LandingScreen,
        events: Vec<Event>,
        screen: Vec2,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let library = &mut Library::default();
        run_at(context, landing, library, events, screen, 0.0, Theme::Dark)
    }

    fn run_at(
        context: &egui::Context,
        landing: &mut LandingScreen,
        library: &mut Library,
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
        let output = context.run_ui(input, |ui| choice = landing.show(ui.ctx(), theme, library));
        (output, choice)
    }

    fn click(
        context: &egui::Context,
        landing: &mut LandingScreen,
        library: &mut Library,
        position: Pos2,
        button: PointerButton,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let press = |pressed| Event::PointerButton {
            pos: position,
            button,
            pressed,
            modifiers: Default::default(),
        };
        run_with(
            context,
            landing,
            library,
            vec![Event::PointerMoved(position), press(true)],
        );
        run_with(context, landing, library, vec![press(false)])
    }

    fn click_text(
        context: &egui::Context,
        landing: &mut LandingScreen,
        library: &mut Library,
        output: &egui::FullOutput,
        text: &str,
        button: PointerButton,
    ) -> (egui::FullOutput, Option<LandingChoice>) {
        let position = text_position(output, text).unwrap_or_else(|| panic!("no {text}"));
        click(context, landing, library, position + vec2(4.0, 4.0), button)
    }

    const SAVED_SPEED: f64 = 4.0;

    fn saved_example() -> World {
        EXAMPLE_CATEGORIES[0].examples[0].spawn().unwrap()
    }

    fn library_with(names: &[&str]) -> Library {
        let mut library = Library::default();
        for name in names {
            let bytes = crate::serialization::save(&saved_example(), 1.0, SAVED_SPEED).unwrap();
            library.create(name, bytes);
        }
        library
    }

    fn escape() -> Event {
        Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }
    }

    fn card_rects(output: &egui::FullOutput) -> Vec<Rect> {
        output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Rect(rect) if rect.fill == DARK.surface => Some(rect.rect),
                _ => None,
            })
            .collect()
    }

    fn text_positions<'a>(
        output: &'a egui::FullOutput,
        text: &'a str,
    ) -> impl Iterator<Item = Pos2> + 'a {
        output
            .shapes
            .iter()
            .filter_map(move |clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => Some(shape.pos),
                _ => None,
            })
    }

    fn text_position(output: &egui::FullOutput, text: &str) -> Option<Pos2> {
        text_positions(output, text).next()
    }

    fn see_all_of(output: &egui::FullOutput, section: &str) -> Pos2 {
        let heading = text_position(output, section).unwrap_or_else(|| panic!("no {section}"));
        text_positions(output, "See all")
            .min_by(|a, b| (a.y - heading.y).abs().total_cmp(&(b.y - heading.y).abs()))
            .unwrap_or_else(|| panic!("no See all next to {section}"))
            + vec2(4.0, 4.0)
    }

    #[test]
    fn clicking_a_card_opens_its_example() {
        let context = egui::Context::default();
        let mut landing = LandingScreen::new(false);
        let (output, _) = run(&context, &mut landing, vec![]);
        let position = text_position(&output, "1. Spikes").unwrap() + vec2(4.0, 4.0);

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
        assert_eq!(world.query::<&LeakyNeuron>().iter().count(), 2);
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
        let window_with_room_for_two_rows = vec2(1280.0, 700.0);
        run_sized(
            &context,
            &mut landing,
            vec![],
            window_with_room_for_two_rows,
        );
        landing.update(1.0 / 60.0, 4);
        let first = &landing.examples_by_category[0][0].preview;
        let last = &landing
            .examples_by_category
            .last()
            .unwrap()
            .last()
            .unwrap()
            .preview;
        assert!(first.on_screen && first.time > 0.0);
        assert!(!last.on_screen && last.time == 0.0);
    }

    #[test]
    fn follows_the_theme() {
        let screen = vec2(1280.0, 800.0);
        for theme in [Theme::Dark, Theme::Light] {
            let context = egui::Context::default();
            let mut landing = LandingScreen::new(false);
            let library = &mut Library::default();
            let (output, _) = run_at(&context, &mut landing, library, vec![], screen, 0.0, theme);
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
        landing.examples_by_category[0][0].preview.mode = PreviewMode::Still;
        for theme in [Theme::Dark, Theme::Light] {
            run_at(
                &context,
                &mut landing,
                &mut Library::default(),
                vec![],
                vec2(1280.0, 800.0),
                0.0,
                theme,
            );
            let preview = &landing.examples_by_category[0][0].preview;
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
                &mut Library::default(),
                vec![],
                screen,
                frame as f64 / 60.0,
                Theme::Dark,
            )
            .0
        };

        let output = next_frame(&mut landing);
        assert!(neuron_opacities(&output).is_empty());
        while !landing.examples_by_category[0][0].preview.warmed_up() {
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
    #[test]
    fn your_simulations_come_first_with_the_newest_after_the_new_card() {
        let context = egui::Context::default();
        let mut library = library_with(&["Older", "Newer"]);
        let mut landing = LandingScreen::new(false);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let position = |text| text_position(&output, text).unwrap();
        assert!(position(YOUR_SIMULATIONS).y < position("Getting started").y);
        let row = ["New simulation", "Newer", "Older"].map(position);
        assert!(row.iter().all(|card| card.y == row[0].y));
        assert!(row[0].x < row[1].x && row[1].x < row[2].x, "{row:?}");
    }

    #[test]
    fn the_new_card_starts_an_empty_simulation() {
        let context = egui::Context::default();
        let mut library = Library::default();
        let mut landing = LandingScreen::new(false);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let primary = PointerButton::Primary;
        let (_, choice) = click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "New simulation",
            primary,
        );
        assert!(matches!(choice, Some(LandingChoice::NewSimulation)));
    }

    #[test]
    fn clicking_a_saved_simulation_opens_it() {
        let context = egui::Context::default();
        let mut library = library_with(&["Mine"]);
        let id = library.simulations()[0].id.clone();
        let mut landing = LandingScreen::new(false);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let primary = PointerButton::Primary;
        let (_, choice) = click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Mine",
            primary,
        );
        let Some(LandingChoice::Saved {
            id: opened,
            world,
            time,
            speed,
        }) = choice
        else {
            panic!("clicking the card did not open the saved simulation");
        };
        assert_eq!(opened, id);
        assert!(time >= 1.0);
        assert_eq!(speed, SAVED_SPEED);
        let neurons = |world: &World| world.query::<&LeakyNeuron>().iter().count();
        assert_eq!(neurons(&world), neurons(&saved_example()));
    }

    #[test]
    fn cards_follow_the_library() {
        let context = egui::Context::default();
        let mut library = library_with(&["Mine", "Other"]);
        let mut landing = LandingScreen::new(false);
        run_with(&context, &mut landing, &mut library, vec![]);
        let id = library.simulations()[0].id.clone();
        library.rename(&id, "Renamed");
        let other = library.simulations()[1].id.clone();
        library.delete(&other);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert!(text_position(&output, "Renamed").is_some());
        assert!(text_position(&output, "Other").is_none());
        assert_eq!(landing.saved.len(), 1);
    }

    #[test]
    fn deleting_asks_first_and_escape_only_closes_the_question() {
        let context = egui::Context::default();
        let mut library = library_with(&["Mine"]);
        let mut landing = LandingScreen::new(true);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let secondary = PointerButton::Secondary;
        let primary = PointerButton::Primary;
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Mine",
            secondary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Delete…",
            primary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert!(text_position(&output, "Delete “Mine”?").is_some());

        let (_, choice) = run_with(&context, &mut landing, &mut library, vec![escape()]);
        assert!(choice.is_none());
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert!(text_position(&output, "Delete “Mine”?").is_none());
        assert_eq!(library.simulations().len(), 1);

        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Mine",
            secondary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Delete…",
            primary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Delete",
            primary,
        );
        assert!(library.simulations().is_empty());
    }

    #[test]
    fn renaming_from_the_card_menu() {
        let context = egui::Context::default();
        let mut library = library_with(&["Mine"]);
        let mut landing = LandingScreen::new(false);
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Mine",
            PointerButton::Secondary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Rename…",
            PointerButton::Primary,
        );
        run_with(&context, &mut landing, &mut library, vec![]);
        let typed = vec![Event::Text(" too".to_owned())];
        run_with(&context, &mut landing, &mut library, typed);
        let enter = Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        };
        run_with(&context, &mut landing, &mut library, vec![enter]);
        assert_eq!(library.simulations()[0].name, "Mine too");
        assert!(!landing.dialogs.is_open());
    }

    #[test]
    fn a_phone_screen_fits_six_cards_in_two_columns() {
        let context = egui::Context::default();
        let names = ["One", "Two", "Three", "Four", "Five", "Six", "Seven"];
        let mut library = library_with(&names);
        let mut landing = LandingScreen::new(false);
        run_with(&context, &mut landing, &mut library, vec![]);
        let longest_date = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .to_utc();
        for card in &mut landing.saved {
            card.modified = longest_date;
        }
        let phone = vec2(390.0, 664.0);
        let (output, _) = run_at(
            &context,
            &mut landing,
            &mut library,
            vec![],
            phone,
            0.0,
            Theme::Dark,
        );
        let mut cards = card_rects(&output);
        cards.sort_by(|a, b| {
            a.top()
                .total_cmp(&b.top())
                .then(a.left().total_cmp(&b.left()))
        });
        let first_six = &cards[..6];
        assert!(
            first_six.iter().all(|card| card.bottom() <= phone.y),
            "{first_six:?}"
        );
        for row in first_six.chunks(2) {
            assert_eq!(row[0].top(), row[1].top());
            assert!(row[0].right() < row[1].left());
        }

        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let shown = names
            .iter()
            .filter(|name| text_position(&output, name).is_some())
            .count();
        assert_eq!(shown, CARDS_PER_SECTION - 1);
        assert!(text_position(&output, "See all").is_some());
    }

    #[test]
    fn see_all_opens_the_whole_category_on_a_page_of_its_own() {
        let context = egui::Context::default();
        let mut library = Library::default();
        let mut landing = LandingScreen::new(true);
        let category = EXAMPLE_CATEGORIES
            .iter()
            .position(|category| category.name == "Circuits")
            .unwrap();
        let names: Vec<&str> = EXAMPLE_CATEGORIES[category]
            .examples
            .iter()
            .map(|example| example.name)
            .collect();
        let last = *names.last().unwrap();
        let shown = |output: &egui::FullOutput| {
            names
                .iter()
                .filter(|name| text_position(output, name).is_some())
                .count()
        };
        let last_preview_runs =
            |landing: &LandingScreen| landing.examples_by_category[category][10].preview.on_screen;
        let primary = PointerButton::Primary;

        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert_eq!(shown(&output), CARDS_PER_SECTION);
        let (_, choice) = click(
            &context,
            &mut landing,
            &mut library,
            see_all_of(&output, "Circuits"),
            primary,
        );
        assert!(choice.is_none());
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert_eq!(shown(&output), names.len());
        assert!(text_position(&output, "Getting started").is_none());
        assert!(last_preview_runs(&landing));

        let (_, choice) = run_with(&context, &mut landing, &mut library, vec![escape()]);
        assert!(choice.is_none());
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert_eq!(shown(&output), CARDS_PER_SECTION);
        assert!(!last_preview_runs(&landing));

        click(
            &context,
            &mut landing,
            &mut library,
            see_all_of(&output, "Circuits"),
            primary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        click_text(
            &context,
            &mut landing,
            &mut library,
            &output,
            "Back",
            primary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        assert!(text_position(&output, "Getting started").is_some());

        click(
            &context,
            &mut landing,
            &mut library,
            see_all_of(&output, "Circuits"),
            primary,
        );
        let (output, _) = run_with(&context, &mut landing, &mut library, vec![]);
        let (_, choice) = click_text(&context, &mut landing, &mut library, &output, last, primary);
        let Some(LandingChoice::Example { name, .. }) = choice else {
            panic!("clicking a card on the page did not open its example");
        };
        assert_eq!(name, last);
    }

    #[test]
    fn edited_times_read_like_a_list_of_notes() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        let at = |date: NaiveDate| date.and_hms_opt(9, 5, 0).unwrap();
        assert_eq!(edited(at(today), today), "Today at 09:05");
        assert_eq!(
            edited(at(today.pred_opt().unwrap()), today),
            "Yesterday at 09:05"
        );
        let earlier = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        assert_eq!(edited(at(earlier), today), "4 March");
        let last_year = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert_eq!(edited(at(last_year), today), "31 December 2025");
    }
}
