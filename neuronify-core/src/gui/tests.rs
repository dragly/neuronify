use egui::{pos2, vec2, Event, PointerButton, Pos2, RawInput};

use super::*;

const SCREEN: Vec2 = vec2(1280.0, 800.0);

fn frame(context: &Context, events: Vec<Event>, show: &mut impl FnMut(&Context)) {
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        events,
        ..Default::default()
    };
    let _ = context.run_ui(input, |ui| show(ui.ctx()));
}

fn appear(context: &Context, show: &mut impl FnMut(&Context)) {
    frame(context, vec![], show);
    frame(context, vec![], show);
}

fn click(context: &Context, position: Pos2, show: &mut impl FnMut(&Context)) {
    let button = |pressed| Event::PointerButton {
        pos: position,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(
        context,
        vec![Event::PointerMoved(position), button(true)],
        show,
    );
    frame(context, vec![button(false)], show);
}

fn corner_button(from_right: usize) -> Pos2 {
    pos2(
        SCREEN.x - MARGIN - FAB_SIZE / 2.0 - from_right as f32 * (FAB_SIZE + FAB_GAP),
        SCREEN.y - MARGIN - FAB_SIZE / 2.0,
    )
}

#[test]
fn the_pencil_switches_editing_on_and_off() {
    let context = Context::default();
    let mut edit_enabled = true;
    let mut show = |context: &Context| {
        corner_buttons(context, Theme::Dark, &mut edit_enabled);
    };
    appear(&context, &mut show);
    click(&context, corner_button(0), &mut show);
    assert!(!edit_enabled);
}

#[test]
fn the_examples_button_asks_for_the_landing_screen() {
    let context = Context::default();
    let mut edit_enabled = true;
    let mut asked = false;
    let mut show = |context: &Context| {
        asked |= corner_buttons(context, Theme::Light, &mut edit_enabled);
    };
    appear(&context, &mut show);
    click(&context, corner_button(1), &mut show);
    assert!(asked && edit_enabled);
}
