use egui::{CursorIcon, Key, Response, RichText, ScrollArea, Sense, Ui, WidgetInfo, WidgetType};

use super::{back, text_button, Colors, PAGE_MARGIN};
use crate::links;
use crate::rendering::Theme;
use crate::third_party::{self, ThirdParty};

const MAX_WIDTH: f32 = 720.0;
const TITLE_SIZE: f32 = 34.0;
const HEADING_SIZE: f32 = 20.0;
const BODY_SIZE: f32 = 15.0;
const DETAIL_SIZE: f32 = 13.5;
const PACKAGE_GAP: f32 = 4.0;
const LICENSE_TEXT_SIZE: f32 = 11.5;
const LICENSE_TEXT_PADDING: i8 = 12;
const LICENSE_TEXT_RADIUS: u8 = 8;
const SECTION_GAP: f32 = 32.0;
const PARAGRAPH_GAP: f32 = 10.0;

pub const SOURCE_CODE: &str = "https://github.com/dragly/neuronify";
pub const PAPER: &str = "https://doi.org/10.1523/ENEURO.0022-17.2017";
const GPL: &str = include_str!("../../../LICENSE");

const INTRO: &str = "Neuronify is an educational tool meant to give intuition for how neurons \
    and neural networks behave.";
const SUPPORT: &str = "Support Neuronify";
const SUPPORT_TEXT: &str = "Neuronify is currently maintained by Svenn-Arne Dragly.";
const SUPPORT_LINK: &str = "How to support its development";
const LICENSE: &str = "License";
const NOTICE: &str = "Neuronify is free software: you can redistribute it and/or modify it under \
    the terms of the GNU General Public License version 3. It comes with no warranty.";
const COPYRIGHT: &str = "Copyright © 2014–2026 the Neuronify developers.";
const SOURCE_LINK: &str = "Source code on GitHub";
const SHOW_LICENSE: &str = "Show the license";
const HIDE_LICENSE: &str = "Hide the license";
const ORIGINAL_PAPER: &str = "Original paper";
const PAPER_TEXT: &str = "Neuronify was first described in an article in eNeuro in 2017. \
    If you use Neuronify in teaching or research, please cite it.";
const PAPER_LINK: &str = "Neuronify: An Educational Simulator for Neural Circuits";
const THIRD_PARTY: &str = "Third-party software";
const THIRD_PARTY_INTRO: &str = "Neuronify is built with open source packages and fonts \
    made by others. Each comes with its own license.";
const SHOW_THIRD_PARTY: &str = "Show the licenses";
const HIDE_THIRD_PARTY: &str = "Hide the licenses";

pub enum AboutChoice {
    Back,
}

pub struct AboutScreen {
    version: &'static str,
    license_shown: bool,
    third_party: Option<Result<ThirdParty, String>>,
    opened: Option<usize>,
}

impl AboutScreen {
    pub fn new(version: &'static str) -> AboutScreen {
        AboutScreen {
            version,
            license_shown: false,
            third_party: None,
            opened: None,
        }
    }

    pub fn show(&mut self, context: &egui::Context, theme: Theme) -> Option<AboutChoice> {
        let colors = Colors::of(theme);
        let mut choice = None;
        #[allow(deprecated)]
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(colors.background))
            .show(context, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width = (ui.available_width() - 2.0 * PAGE_MARGIN).min(MAX_WIDTH);
                        ui.horizontal_top(|ui| {
                            ui.add_space((ui.available_width() - width) / 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                if self.contents(ui, colors) {
                                    choice = Some(AboutChoice::Back);
                                }
                            });
                        });
                    });
            });
        if context.input(|input| input.key_pressed(Key::Escape)) {
            choice = Some(AboutChoice::Back);
        }
        choice
    }

    fn contents(&mut self, ui: &mut Ui, colors: &Colors) -> bool {
        ui.add_space(16.0);
        let back = back(ui, colors).clicked();
        ui.add_space(4.0);
        ui.label(
            RichText::new("Neuronify")
                .size(TITLE_SIZE)
                .color(colors.text),
        );
        ui.label(
            RichText::new(format!("Version {}", self.version))
                .size(BODY_SIZE)
                .color(colors.subtext),
        );
        ui.add_space(PARAGRAPH_GAP);
        paragraph(ui, colors, INTRO);

        heading(ui, colors, SUPPORT);
        paragraph(ui, colors, SUPPORT_TEXT);
        link(ui, colors, SUPPORT_LINK, links::SUPPORT);

        heading(ui, colors, LICENSE);
        paragraph(ui, colors, NOTICE);
        paragraph(ui, colors, COPYRIGHT);
        link(ui, colors, SOURCE_LINK, SOURCE_CODE);
        ui.add_space(PARAGRAPH_GAP);
        let label = if self.license_shown {
            HIDE_LICENSE
        } else {
            SHOW_LICENSE
        };
        if text_button(ui, colors, label).clicked() {
            self.license_shown = !self.license_shown;
        }
        if self.license_shown {
            license_text(ui, colors, GPL);
        }

        heading(ui, colors, ORIGINAL_PAPER);
        paragraph(ui, colors, PAPER_TEXT);
        link(ui, colors, PAPER_LINK, PAPER);

        heading(ui, colors, THIRD_PARTY);
        paragraph(ui, colors, THIRD_PARTY_INTRO);
        let label = if self.third_party.is_some() {
            HIDE_THIRD_PARTY
        } else {
            SHOW_THIRD_PARTY
        };
        if text_button(ui, colors, label).clicked() {
            self.third_party = match self.third_party {
                Some(_) => None,
                None => Some(third_party::load()),
            };
            self.opened = None;
        }
        match &self.third_party {
            Some(Ok(third_party)) => {
                ui.add_space(PARAGRAPH_GAP);
                for (index, package) in third_party.packages.iter().enumerate() {
                    let opened = self.opened == Some(index);
                    if package_row(ui, colors, &package.title(), &package.license).clicked() {
                        self.opened = (!opened).then_some(index);
                    }
                    if opened {
                        for text in package
                            .texts
                            .iter()
                            .filter_map(|&text| third_party.texts.get(text))
                        {
                            license_text(ui, colors, text);
                        }
                    }
                }
            }
            Some(Err(error)) => {
                ui.add_space(PARAGRAPH_GAP);
                paragraph(ui, colors, error);
            }
            None => {}
        }
        ui.add_space(40.0);
        back
    }
}

fn heading(ui: &mut Ui, colors: &Colors, text: &str) {
    ui.add_space(SECTION_GAP);
    ui.label(RichText::new(text).size(HEADING_SIZE).color(colors.text));
    ui.add_space(PARAGRAPH_GAP);
}

fn paragraph(ui: &mut Ui, colors: &Colors, text: &str) {
    ui.label(RichText::new(text).size(BODY_SIZE).color(colors.text));
    ui.add_space(PARAGRAPH_GAP);
}

fn link(ui: &mut Ui, colors: &Colors, text: &str, url: &str) {
    let text = RichText::new(text).size(BODY_SIZE).color(colors.accent);
    links::open_on_click(&ui.add(egui::Link::new(text)), url);
}

fn package_row(ui: &mut Ui, colors: &Colors, title: &str, license: &str) -> Response {
    let rect = ui
        .vertical(|ui| {
            ui.add_space(PACKAGE_GAP);
            ui.label(RichText::new(title).size(BODY_SIZE).color(colors.text));
            ui.label(
                RichText::new(license)
                    .size(DETAIL_SIZE)
                    .color(colors.subtext),
            );
            ui.add_space(PACKAGE_GAP);
        })
        .response
        .rect;
    let response = ui.interact(rect, ui.id().with(title), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, title));
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn license_text(ui: &mut Ui, colors: &Colors, text: &str) {
    ui.add_space(PARAGRAPH_GAP);
    egui::Frame::NONE
        .fill(colors.surface)
        .corner_radius(LICENSE_TEXT_RADIUS)
        .inner_margin(LICENSE_TEXT_PADDING)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(text)
                    .monospace()
                    .size(LICENSE_TEXT_SIZE)
                    .color(colors.subtext),
            );
        });
    ui.add_space(PARAGRAPH_GAP);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{
        pos2, vec2, Event, FullOutput, OutputCommand, PointerButton, RawInput, Rect, Shape,
    };

    struct Page {
        context: egui::Context,
        about: AboutScreen,
        choice: Option<AboutChoice>,
        output: FullOutput,
    }

    impl Page {
        fn open() -> Page {
            let mut page = Page {
                context: egui::Context::default(),
                about: AboutScreen::new("2.0.0"),
                choice: None,
                output: FullOutput::default(),
            };
            page.frame(vec![]);
            page
        }

        fn frame(&mut self, events: Vec<Event>) {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 4000.0))),
                events,
                ..Default::default()
            };
            let Page { about, choice, .. } = self;
            self.output = self.context.run_ui(input, |ui| {
                *choice = about.show(ui.ctx(), Theme::Dark);
            });
        }

        fn texts(&self) -> impl Iterator<Item = (&str, Rect)> {
            self.output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    Shape::Text(shape) => Some((
                        shape.galley.text(),
                        Rect::from_min_size(shape.pos, shape.galley.size()),
                    )),
                    _ => None,
                })
        }

        fn shows(&self, text: &str) -> bool {
            self.texts().any(|(shown, _)| shown == text)
        }

        fn click_text(&mut self, text: &str) {
            let position = self
                .texts()
                .find(|(shown, _)| *shown == text)
                .unwrap_or_else(|| panic!("{text} is not on screen"))
                .1
                .center();
            let button = |pressed| Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            self.frame(vec![Event::PointerMoved(position), button(true)]);
            self.frame(vec![button(false)]);
        }

        fn opened_url(&self) -> Option<&str> {
            self.output
                .platform_output
                .commands
                .iter()
                .find_map(|command| match command {
                    OutputCommand::OpenUrl(open) => Some(open.url.as_str()),
                    _ => None,
                })
        }
    }

    #[test]
    fn shows_the_version_it_was_given() {
        assert!(Page::open().shows("Version 2.0.0"));
    }

    #[test]
    fn back_and_escape_leave_the_page() {
        let mut page = Page::open();
        page.click_text("Back");
        assert!(matches!(page.choice, Some(AboutChoice::Back)));

        let mut page = Page::open();
        page.frame(vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }]);
        assert!(matches!(page.choice, Some(AboutChoice::Back)));
    }

    #[test]
    fn the_license_text_shows_on_request() {
        let mut page = Page::open();
        assert!(!page.shows(GPL));
        page.click_text(SHOW_LICENSE);
        page.frame(vec![]);
        assert!(page.shows(GPL));
        page.click_text(HIDE_LICENSE);
        assert!(!page.shows(GPL));
    }

    #[test]
    fn the_third_party_licenses_open_one_package_at_a_time() {
        let third_party = third_party::load().unwrap();
        let first = &third_party.packages[0];
        let second = &third_party.packages[1];
        let mut page = Page::open();
        assert!(!page.shows(&first.title()));
        page.click_text(SHOW_THIRD_PARTY);
        page.frame(vec![]);
        assert!(page.shows(&first.title()));
        assert!(page.shows(&first.license));

        page.click_text(&first.title());
        page.frame(vec![]);
        assert!(page.shows(&third_party.texts[first.texts[0]]));
        page.click_text(&second.title());
        page.frame(vec![]);
        assert!(!page.shows(&third_party.texts[first.texts[0]]));
        assert!(page.shows(&third_party.texts[second.texts[0]]));

        page.click_text(HIDE_THIRD_PARTY);
        page.frame(vec![]);
        assert!(!page.shows(&first.title()));
    }

    #[test]
    fn links_open_the_source_code_and_the_paper() {
        let mut page = Page::open();
        page.click_text(SOURCE_LINK);
        assert_eq!(page.opened_url(), Some(SOURCE_CODE));
        page.click_text(PAPER_LINK);
        assert_eq!(page.opened_url(), Some(PAPER));
    }

    #[test]
    fn the_support_section_links_to_the_support_page() {
        let mut page = Page::open();
        assert!(page.shows(SUPPORT_TEXT));
        page.click_text(SUPPORT_LINK);
        assert_eq!(page.opened_url(), Some(links::SUPPORT));
    }
}
