use egui::{pos2, vec2, Event, FullOutput, PointerButton, Pos2, RawInput, Shape};
use glam::Vec3;
use hecs::Entity;

use super::*;
use crate::components::LeakyNeuron;
use crate::constants::DEFAULT_SPEED;
use crate::elements::Element;
use crate::tools::Tool;

const SCREEN: Vec2 = vec2(1280.0, 800.0);
const PHONE: Vec2 = vec2(390.0, 760.0);

struct Harness {
    context: Context,
    screen: Vec2,
    gui: Gui,
    edit_enabled: bool,
    editor: Editor,
    world: World,
    theme: Theme,
    running: bool,
    speed: f64,
    choice: Option<MenuChoice>,
    output: Option<FullOutput>,
    time: f64,
    saving: Saving,
}

impl Harness {
    fn new(edit_enabled: bool) -> Harness {
        Harness::with(edit_enabled, World::new(), None)
    }

    fn with(edit_enabled: bool, world: World, selection: Option<Entity>) -> Harness {
        Harness::sized(SCREEN, edit_enabled, world, selection)
    }

    fn phone(edit_enabled: bool, world: World, selection: Option<Entity>) -> Harness {
        Harness::sized(PHONE, edit_enabled, world, selection)
    }

    fn sized(screen: Vec2, edit_enabled: bool, world: World, selection: Option<Entity>) -> Harness {
        let mut harness = Harness {
            context: Context::default(),
            screen,
            gui: Gui::default(),
            edit_enabled,
            editor: Editor::default(),
            world,
            theme: Theme::Dark,
            running: true,
            speed: DEFAULT_SPEED,
            choice: None,
            output: None,
            time: 0.0,
            saving: Saving::Save,
        };
        harness.editor.selection = selection;
        for _ in 0..3 {
            harness.frame(vec![]);
        }
        harness
    }

    fn frame(&mut self, events: Vec<Event>) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.screen)),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let Harness {
            context,
            gui,
            edit_enabled,
            editor,
            world,
            theme,
            running,
            speed,
            choice,
            time,
            saving,
            ..
        } = self;
        let output = context.run_ui(input, |ui| {
            let controls = Controls {
                edit_enabled,
                editor,
                world,
                theme,
                time: *time,
                running,
                speed,
                fps: 60.0,
                saving: *saving,
            };
            if let Some(picked) = gui.show(ui.ctx(), controls) {
                *choice = Some(picked);
            }
        });
        self.output = Some(output);
        self.time += 0.1;
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

    fn drag(&mut self, from: Pos2, to: Pos2) {
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        self.frame(vec![Event::PointerMoved(from), button(from, true)]);
        for step in 1..=4 {
            self.frame(vec![Event::PointerMoved(
                from + (to - from) * step as f32 / 4.0,
            )]);
        }
        self.frame(vec![button(to, false)]);
        for _ in 0..5 {
            self.frame(vec![]);
        }
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

    fn click_button(&mut self, id: &str) {
        let response = self
            .context
            .read_response(Id::new(id))
            .unwrap_or_else(|| panic!("{id} is not on screen"));
        self.click(response.rect.center());
    }

    fn press(&mut self, modifiers: Modifiers, key: Key) {
        self.frame(
            [true, false]
                .map(|pressed| Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                })
                .into(),
        );
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
fn the_neurons_flyout_chooses_between_point_and_compartmental_neurons() {
    let mut harness = Harness::new(true);
    harness.click_text("Neurons");
    assert!(harness.shows("Drag from a neuron to connect it to another."));
    harness.click_text("Compartmental neurons");
    assert!(harness.editor.compartments);
    assert!(harness.shows("Drag from a neuron or dendrite to grow an axon."));
    harness.click_text("Point neurons");
    assert!(!harness.editor.compartments);
}

#[test]
fn the_inputs_say_how_to_connect_them() {
    let hint = "Press and drag to place an input and connect it to a neuron.";
    let mut harness = Harness::new(true);
    harness.click_text("Inputs");
    assert!(harness.shows(hint));

    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Add");
    let height = |harness: &Harness, text: &str| harness.text_position(text).unwrap().y;
    assert!(height(&harness, "INPUTS") < height(&harness, hint));
    assert!(height(&harness, hint) < height(&harness, "Current source"));
}

#[test]
fn the_menu_switches_firing_rates_on_and_voltages_off() {
    use crate::components::{NeuronDisplay, Setting};
    let mut harness = Harness::new(true);
    harness.click_text("Menu");
    harness.click_text("Show firing rate");
    harness.click_text("Show voltage");
    let display = NeuronDisplay::of(&harness.world);
    assert!(display.firing_rate);
    assert!(!display.voltage);
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

#[test]
fn the_menu_asks_to_open_and_save() {
    for (row, asked) in [("Open…", MenuChoice::Open), ("Save…", MenuChoice::Save)] {
        let mut harness = Harness::new(false);
        harness.click(pos2(MARGIN + FAB_SIZE / 2.0, MARGIN + FAB_SIZE / 2.0));
        harness.click_text(row);
        assert_eq!(harness.choice, Some(asked));
    }
}

#[test]
fn the_menu_offers_to_share_where_the_browser_can() {
    for (saving, save, share) in [
        (Saving::Save, true, false),
        (Saving::SaveOrShare, true, true),
        (Saving::Share, false, true),
    ] {
        let mut harness = Harness::new(false);
        harness.saving = saving;
        harness.click(pos2(MARGIN + FAB_SIZE / 2.0, MARGIN + FAB_SIZE / 2.0));
        assert_eq!(harness.shows("Save…"), save);
        assert_eq!(harness.shows("Share…"), share);
        if share {
            harness.click_text("Share…");
            assert_eq!(harness.choice, Some(MenuChoice::Share));
        }
    }
}

fn one_neuron() -> (World, Entity) {
    let mut world = World::new();
    let neuron = Element::AdaptiveNeuron.spawn(&mut world, Vec3::ZERO);
    (world, neuron)
}

#[test]
fn the_inspector_shows_the_selection() {
    for edit_enabled in [true, false] {
        let (world, neuron) = one_neuron();
        let harness = Harness::with(edit_enabled, world, Some(neuron));
        assert!(harness.shows("Adaptive neuron"));
        assert!(harness.shows("Threshold"));
    }
    assert!(!Harness::new(true).shows("Threshold"));
}

#[test]
fn the_inspector_deletes_the_selection() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::with(true, world, Some(neuron));
    harness.click_button(inspector::DELETE);
    assert!(!harness.world.contains(neuron));
    assert_eq!(harness.editor.selection, None);
    assert!(!harness.shows("Threshold"));
}

#[test]
fn undo_and_redo_what_the_inspector_deleted() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::with(true, world, Some(neuron));
    harness.click_button(inspector::DELETE);
    assert!(!harness.world.contains(neuron));
    harness.click(corner_button(3));
    assert!(harness.world.contains(neuron));
    harness.click(corner_button(2));
    assert!(!harness.world.contains(neuron));
}

#[test]
fn ctrl_z_undoes_and_ctrl_shift_z_or_ctrl_y_redoes() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::with(true, world, Some(neuron));
    harness.editor.erase_selection(&mut harness.world);
    harness.press(Modifiers::COMMAND, Key::Z);
    assert!(harness.world.contains(neuron));
    harness.press(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    assert!(!harness.world.contains(neuron));
    harness.press(Modifiers::COMMAND, Key::Z);
    harness.press(Modifiers::COMMAND, Key::Y);
    assert!(!harness.world.contains(neuron));
}

#[test]
fn ctrl_z_is_left_to_the_text_of_a_note() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, Vec3::ZERO);
    let mut harness = Harness::with(true, world, Some(note));
    let neuron = Element::ExcitatoryNeuron.spawn(&mut harness.world, Vec3::X * 10.0);
    harness.editor.selection = Some(neuron);
    harness.editor.erase_selection(&mut harness.world);
    harness.editor.selection = Some(note);
    harness.frame(vec![]);
    harness.click_text("New note");
    harness.press(Modifiers::COMMAND, Key::Z);
    assert!(!harness.world.contains(neuron));
}

#[test]
fn letters_choose_tools_and_step_through_a_group() {
    let mut harness = Harness::new(true);
    harness.press(Modifiers::NONE, Key::N);
    assert_eq!(harness.editor.tool, Tool::Place(Element::ExcitatoryNeuron));
    harness.press(Modifiers::NONE, Key::N);
    assert_eq!(harness.editor.tool, Tool::Place(Element::InhibitoryNeuron));
    harness.press(Modifiers::NONE, Key::C);
    assert_eq!(harness.editor.tool, Tool::StaticConnection);
    harness.press(Modifiers::NONE, Key::N);
    assert_eq!(harness.editor.tool, Tool::Place(Element::InhibitoryNeuron));
    harness.press(Modifiers::SHIFT, Key::E);
    assert_eq!(harness.editor.tool, Tool::Place(Element::InhibitoryNeuron));
    harness.press(Modifiers::NONE, Key::V);
    assert_eq!(harness.editor.tool, Tool::Select);
}

#[test]
fn tool_letters_are_left_to_the_text_of_a_note() {
    let mut world = World::new();
    let note = Element::Note.spawn(&mut world, Vec3::ZERO);
    let mut harness = Harness::with(true, world, Some(note));
    harness.click_text("New note");
    harness.press(Modifiers::NONE, Key::N);
    assert_eq!(harness.editor.tool, Tool::Select);
}

#[test]
fn dragging_a_slider_is_one_step_to_undo() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::with(true, world, Some(neuron));
    let threshold =
        |harness: &Harness| harness.world.get::<&LeakyNeuron>(neuron).unwrap().threshold;
    let original = threshold(&harness);
    let slider = harness.text_position("Threshold").unwrap() + vec2(0.0, 22.0);
    harness.drag(slider, slider + vec2(120.0, 0.0));
    assert_ne!(threshold(&harness), original);

    harness.click(corner_button(3));
    assert_eq!(threshold(&harness), original);
    assert!(!harness.editor.can_undo());
}

#[test]
fn closing_the_inspector_clears_the_selection() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::with(true, world, Some(neuron));
    harness.click_button(inspector::CLOSE);
    assert!(harness.world.contains(neuron));
    assert_eq!(harness.editor.selection, None);
}

#[test]
fn the_inspector_forgets_a_selection_that_was_erased() {
    let (mut world, neuron) = one_neuron();
    let other = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::X * 10.0);
    world.despawn(neuron).unwrap();
    let harness = Harness::with(true, world, Some(neuron));
    assert!(!harness.shows("Threshold"));
    assert!(harness.world.contains(other));
}

#[test]
fn the_playback_bar_pauses_and_resumes() {
    for edit_enabled in [true, false] {
        let mut harness = Harness::new(edit_enabled);
        harness.click_button(playback::PLAY);
        assert!(!harness.running);
        harness.click_button(playback::PLAY);
        assert!(harness.running);
    }
}

#[test]
fn the_playback_bar_keeps_the_speed_within_bounds() {
    let mut harness = Harness::new(true);
    harness.speed = 4.0;
    harness.click_button(playback::FASTER);
    assert_eq!(harness.speed, 8.0);
    harness.click_button(playback::FASTER);
    assert_eq!(harness.speed, 8.0);
    harness.speed = 1.0;
    harness.click_button(playback::SLOWER);
    assert_eq!(harness.speed, 0.5);
    harness.click_button(playback::SLOWER);
    assert_eq!(harness.speed, 0.5);
}

#[test]
fn the_canvas_area_leaves_out_the_rail_and_the_playback_bar() {
    let screen = Rect::from_min_size(Pos2::ZERO, SCREEN);
    let editing = canvas_area(screen, true);
    let viewing = canvas_area(screen, false);
    assert_eq!(editing.left(), MARGIN + RAIL_WIDTH);
    assert_eq!(viewing.left(), 0.0);
    assert_eq!(editing.bottom(), SCREEN.y - MARGIN - BAR_HEIGHT);
    assert_eq!(editing.right(), SCREEN.x);
}

fn top_corner(right: bool) -> Pos2 {
    let inset = MARGIN + FAB_SIZE / 2.0;
    pos2(if right { PHONE.x - inset } else { inset }, inset)
}

#[test]
fn phones_get_a_tool_bar_instead_of_the_rail() {
    for edit_enabled in [true, false] {
        let harness = Harness::phone(edit_enabled, World::new(), None);
        assert!(harness.shows("Stimulate") && harness.shows("Add") && harness.shows("Playback"));
        assert!(!harness.shows("Neurons") && !harness.shows("×4"));
    }
}

#[test]
fn a_tool_on_the_tool_bar_starts_editing() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Erase");
    assert_eq!(harness.editor.tool, Tool::Erase);
    assert!(harness.edit_enabled);
}

#[test]
fn the_add_sheet_starts_editing_with_what_to_place() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Add");
    harness.click_text("Poisson generator");
    assert_eq!(harness.editor.tool, Tool::Place(Element::PoissonGenerator));
    assert!(harness.edit_enabled);
    assert!(!harness.shows("Poisson generator"));
}

#[test]
fn the_add_sheet_chooses_the_kind_of_neuron_right_under_the_neurons() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Add");
    let height = |harness: &Harness, text: &str| harness.text_position(text).unwrap().y;
    let choice = height(&harness, "Compartmental neurons");
    assert!(height(&harness, "NEURONS") < choice);
    assert!(choice < height(&harness, "Excitatory neuron"));
    harness.click_text("Compartmental neurons");
    assert!(harness.editor.compartments);
    assert!(harness.shows("Excitatory neuron"));
}

#[test]
fn the_add_sheet_closes_when_pulled_down() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Add");
    let tile = harness.text_position("Poisson generator").unwrap();
    let sheet = harness
        .context
        .memory(|memory| memory.area_rect(Id::new("add_sheet")))
        .unwrap();
    let handle = sheet.center_top() + vec2(0.0, 10.0);
    harness.drag(handle, handle + vec2(0.0, 40.0));
    assert_eq!(harness.text_position("Poisson generator"), Some(tile));
    harness.drag(handle, handle + vec2(0.0, 500.0));
    assert!(!harness.shows("Poisson generator"));
}

#[test]
fn done_stops_editing_on_phones() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::phone(true, world, Some(neuron));
    harness.editor.tool = Tool::Erase;
    harness.click(top_corner(true));
    assert!(!harness.edit_enabled);
    assert_eq!(harness.editor.selection, None);
    assert_eq!(harness.editor.tool, Tool::Select);
}

#[test]
fn undo_and_redo_sit_at_the_top_on_phones() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::phone(true, world, Some(neuron));
    let slot = |from_right: f32| top_corner(true) - vec2(from_right * (FAB_SIZE + FAB_GAP), 0.0);
    harness.editor.erase_selection(&mut harness.world);
    harness.frame(vec![]);
    assert!(!harness.world.contains(neuron));
    harness.click(slot(3.0));
    assert!(harness.world.contains(neuron));
    harness.click(slot(2.0));
    assert!(!harness.world.contains(neuron));
}

#[test]
fn the_more_menu_sits_at_the_top_right_on_phones() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click(top_corner(true));
    harness.click_text("Light");
    assert_eq!(harness.theme, Theme::Light);
}

#[test]
fn the_playback_bar_hides_behind_a_button_on_phones() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click_text("Playback");
    assert!(harness.shows("×2"));
    harness.frame(vec![]);
    harness.click_button(playback::PLAY);
    assert!(!harness.running);
}

#[test]
fn selecting_starts_editing_on_phones() {
    let (world, neuron) = one_neuron();
    let harness = Harness::phone(false, world, Some(neuron));
    assert!(harness.edit_enabled);
    assert!(harness.shows("Threshold"));
}

#[test]
fn the_examples_button_sits_at_the_top_left_on_phones() {
    let mut harness = Harness::phone(false, World::new(), None);
    harness.click(top_corner(false));
    assert!(matches!(harness.choice, Some(MenuChoice::Examples)));
}

#[test]
fn the_inspector_is_a_sheet_on_phones() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::phone(true, world, Some(neuron));
    let close = harness
        .context
        .read_response(Id::new(inspector::CLOSE))
        .unwrap();
    assert!(close.rect.top() > PHONE.y / 2.0);
    harness.click_button(inspector::CLOSE);
    assert_eq!(harness.editor.selection, None);
}

#[test]
fn the_inspector_sheet_keeps_its_height_for_a_new_selection_only() {
    let (mut world, neuron) = one_neuron();
    let other = Element::InhibitoryNeuron.spawn(&mut world, Vec3::X * 10.0);
    let mut harness = Harness::phone(true, world, Some(neuron));
    let title = |harness: &Harness| {
        ["Adaptive neuron", "Inhibitory neuron"]
            .into_iter()
            .find_map(|name| harness.text_position(name))
            .map(|position| position.y)
    };
    let peeking = title(&harness).unwrap();
    harness.click(pos2(PHONE.x / 2.0, peeking - 30.0));
    for _ in 0..5 {
        harness.frame(vec![]);
    }
    let expanded = title(&harness).unwrap();
    assert!(expanded < peeking - 300.0, "{peeking} {expanded}");

    harness.editor.selection = Some(other);
    for _ in 0..4 {
        harness.frame(vec![]);
        assert_eq!(title(&harness), Some(expanded));
    }

    harness.editor.selection = None;
    harness.frame(vec![]);
    harness.frame(vec![]);
    harness.editor.selection = Some(neuron);
    for _ in 0..4 {
        harness.frame(vec![]);
        assert_eq!(title(&harness), Some(peeking));
    }
}

#[test]
fn the_inspector_sheet_follows_its_handle() {
    let (world, neuron) = one_neuron();
    let mut harness = Harness::phone(true, world, Some(neuron));
    let close_top = |harness: &Harness| {
        harness
            .context
            .read_response(Id::new(inspector::CLOSE))
            .unwrap()
            .rect
            .top()
    };
    let peeking = close_top(&harness);
    let handle = |harness: &Harness| pos2(PHONE.x / 2.0, close_top(harness) - 15.0);
    harness.drag(handle(&harness), handle(&harness) - vec2(0.0, 300.0));
    let expanded = close_top(&harness);
    assert!(expanded < peeking - 300.0, "{peeking} {expanded}");
    harness.drag(handle(&harness), handle(&harness) + vec2(0.0, 300.0));
    assert_eq!(close_top(&harness), peeking);
    harness.drag(handle(&harness), handle(&harness) + vec2(0.0, 150.0));
    assert_eq!(harness.editor.selection, None);
}

#[test]
fn the_canvas_area_on_phones_leaves_out_the_bars() {
    let screen = Rect::from_min_size(Pos2::ZERO, PHONE);
    for edit_enabled in [true, false] {
        let area = canvas_area(screen, edit_enabled);
        assert_eq!(area.left(), 0.0);
        assert_eq!(area.top(), MARGIN + FAB_SIZE);
        assert_eq!(area.bottom(), PHONE.y - TOOL_BAR_MARGIN - TOOL_BAR_HEIGHT);
    }
}
