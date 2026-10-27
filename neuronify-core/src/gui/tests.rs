use egui::{pos2, vec2, Event, FullOutput, PointerButton, Pos2, RawInput, Shape};

use super::*;
use crate::elements::Element;
use crate::tools::Tool;

const SCREEN: Vec2 = vec2(1280.0, 800.0);

struct Harness {
    context: Context,
    gui: Gui,
    edit_enabled: bool,
    editor: Editor,
    theme: Theme,
    choice: Option<MenuChoice>,
    output: Option<FullOutput>,
    time: f64,
}

impl Harness {
    fn new(edit_enabled: bool) -> Harness {
        let mut harness = Harness {
            context: Context::default(),
            gui: Gui::default(),
            edit_enabled,
            editor: Editor::default(),
            theme: Theme::Dark,
            choice: None,
            output: None,
            time: 0.0,
        };
        harness.frame(vec![]);
        harness.frame(vec![]);
        harness
    }

    fn frame(&mut self, events: Vec<Event>) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        self.time += 0.1;
        let Harness {
            context,
            gui,
            edit_enabled,
            editor,
            theme,
            choice,
            ..
        } = self;
        let output = context.run_ui(input, |ui| {
            let controls = Controls {
                edit_enabled,
                editor,
                theme,
                fps: 60.0,
            };
            if let Some(picked) = gui.show(ui.ctx(), controls) {
                *choice = Some(picked);
            }
        });
        self.output = Some(output);
    }

    fn click(&mut self, position: Pos2) {
        let button = |pressed| Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        self.frame(vec![Event::PointerMoved(position), button(true)]);
        self.frame(vec![button(false)]);
        self.frame(vec![]);
    }

    fn text_position(&self, text: &str) -> Option<Pos2> {
        self.output
            .as_ref()?
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => {
                    Some(shape.pos + shape.galley.size() / 2.0)
                }
                _ => None,
            })
    }

    fn shows(&self, text: &str) -> bool {
        self.text_position(text).is_some()
    }

    fn click_text(&mut self, text: &str) {
        let position = self
            .text_position(text)
            .unwrap_or_else(|| panic!("{text} is not on screen"));
        self.click(position);
    }
}

fn corner_button(from_right: usize) -> Pos2 {
    pos2(
        SCREEN.x - MARGIN - FAB_SIZE / 2.0 - from_right as f32 * (FAB_SIZE + FAB_GAP),
        SCREEN.y - MARGIN - FAB_SIZE / 2.0,
    )
}

#[test]
fn the_pencil_switches_editing_on_and_off() {
    let mut harness = Harness::new(true);
    harness.click(corner_button(0));
    assert!(!harness.edit_enabled);
    harness.click(corner_button(0));
    assert!(harness.edit_enabled);
}

#[test]
fn the_examples_button_asks_for_the_landing_screen() {
    let mut harness = Harness::new(true);
    harness.click(corner_button(1));
    assert!(matches!(harness.choice, Some(MenuChoice::Examples)));
    assert!(harness.edit_enabled);
}

#[test]
fn the_rail_shows_only_while_editing() {
    assert!(Harness::new(true).shows("Neurons"));
    assert!(!Harness::new(false).shows("Neurons"));
}

#[test]
fn a_tool_on_the_rail_is_chosen_with_one_click() {
    let mut harness = Harness::new(true);
    harness.click_text("Erase");
    assert_eq!(harness.editor.tool, Tool::Erase);
}

#[test]
fn a_group_opens_a_flyout_to_choose_from() {
    let mut harness = Harness::new(true);
    assert!(!harness.shows("Inhibitory neuron"));
    harness.click_text("Neurons");
    harness.click_text("Inhibitory neuron");
    assert_eq!(harness.editor.tool, Tool::Place(Element::InhibitoryNeuron));
    assert!(!harness.shows("Inhibitory neuron"));
}

#[test]
fn pressing_outside_closes_the_flyout() {
    let mut harness = Harness::new(true);
    harness.click_text("Inputs");
    assert!(harness.shows("Touch sensor"));
    harness.click(pos2(700.0, 400.0));
    assert!(!harness.shows("Touch sensor"));
    assert_eq!(harness.editor.tool, Tool::Select);
}

#[test]
fn the_neurons_flyout_decides_whether_new_neurons_get_dendrites() {
    let mut harness = Harness::new(true);
    harness.click_text("Neurons");
    harness.click_text("New neurons get dendrites");
    assert!(harness.editor.dendrites);
}

#[test]
fn the_menu_switches_the_theme() {
    let mut harness = Harness::new(true);
    harness.click_text("Menu");
    harness.click_text("Light");
    assert_eq!(harness.theme, Theme::Light);
}

#[test]
fn the_menu_opens_the_examples() {
    let mut harness = Harness::new(false);
    harness.click(pos2(MARGIN + FAB_SIZE / 2.0, MARGIN + FAB_SIZE / 2.0));
    harness.click_text("Examples");
    assert!(matches!(harness.choice, Some(MenuChoice::Examples)));
    assert!(!harness.shows("Examples"));
}
