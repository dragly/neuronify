use chrono::{DateTime, Utc};
use egui::{pos2, Color32, Painter, Pos2, Rect, TextureHandle, TextureOptions};
use hecs::World;

use crate::constants::{DEFAULT_SPEED, SIMULATION_STEPS_PER_SECOND, VOLTMETER_TIME_WINDOW};
use crate::frame_clock::StepClock;
use crate::landing::raster::{tessellate_untextured_in_pixels, UnfinishedDrawing};
use crate::landing::top_down::top_down_shapes;
use crate::rendering::{circuit_bounds, Bounds, Palette, Theme};
use crate::simulation;

pub const THUMBNAIL_PADDING: f32 = 10.0;
const WARM_UP_STEPS_PER_FRAME: u32 = 64;
pub const PREVIEW_FADE_IN_SECONDS: f64 = 0.25;
const MAX_LIVE_PREVIEW_ENTITIES: u32 = 1000;
const STEPS_UNTIL_VOLTMETERS_ARE_DRAWN: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreviewMode {
    Live,
    Still,
}

fn preview_mode(world: &World) -> PreviewMode {
    if world.len() <= MAX_LIVE_PREVIEW_ENTITIES {
        PreviewMode::Live
    } else {
        PreviewMode::Still
    }
}

pub struct Preview {
    pub world: World,
    pub time: f64,
    simulation_clock: StepClock,
    bounds: Option<Bounds>,
    pub mode: PreviewMode,
    pub on_screen: bool,
    fade_in_started_at: Option<f64>,
    pub still_drawing: Option<StillDrawing>,
}

impl Preview {
    pub fn new(mut world: World, mut time: f64) -> Preview {
        let mode = preview_mode(&world);
        if mode == PreviewMode::Still {
            simulation::step_simulation(
                &mut world,
                &mut time,
                STEPS_UNTIL_VOLTMETERS_ARE_DRAWN,
                None,
            );
        }
        Preview {
            bounds: circuit_bounds(&world),
            mode,
            world,
            time,
            simulation_clock: StepClock::default(),
            on_screen: false,
            fade_in_started_at: None,
            still_drawing: None,
        }
    }

    pub fn warmed_up(&self) -> bool {
        self.time >= VOLTMETER_TIME_WINDOW
    }

    pub fn step(&mut self, seconds: f64, physics_steps: u32) {
        if self.mode == PreviewMode::Still || !self.on_screen {
            return;
        }
        let simulation_steps = if self.warmed_up() {
            self.simulation_clock
                .steps(seconds, DEFAULT_SPEED * SIMULATION_STEPS_PER_SECOND)
        } else {
            WARM_UP_STEPS_PER_FRAME
        };
        simulation::step_simulation(&mut self.world, &mut self.time, simulation_steps, None);
        simulation::step_physics(&mut self.world, physics_steps);
    }

    pub fn paint(
        &mut self,
        painter: &Painter,
        thumbnail: Rect,
        theme: Theme,
        still_drawing_deadline: &mut Option<DateTime<Utc>>,
    ) {
        match self.mode {
            PreviewMode::Live => paint_live_preview(painter, thumbnail, self, &theme.palette()),
            PreviewMode::Still => {
                paint_still_preview(painter, thumbnail, self, theme, still_drawing_deadline)
            }
        }
    }
}

pub struct StillDrawing {
    size_in_pixels: [usize; 2],
    pub theme: Theme,
    texture: TextureHandle,
    unfinished: Option<UnfinishedDrawing>,
}

impl StillDrawing {
    fn is_for(&self, size_in_pixels: [usize; 2], theme: Theme) -> bool {
        self.size_in_pixels == size_in_pixels && self.theme == theme
    }
}

fn paint_live_preview(
    painter: &Painter,
    thumbnail: Rect,
    preview: &mut Preview,
    palette: &Palette,
) {
    let Some(bounds) = &preview.bounds else {
        return;
    };
    if !preview.warmed_up() {
        return;
    }
    let now = painter.ctx().input(|input| input.time);
    let fade_in_started_at = *preview.fade_in_started_at.get_or_insert(now);
    let opacity = ((now - fade_in_started_at) / PREVIEW_FADE_IN_SECONDS).min(1.0) as f32;
    if opacity > 0.0 {
        let mut painter = painter.clone();
        painter.multiply_opacity(opacity);
        painter.extend(top_down_shapes(
            thumbnail.shrink(THUMBNAIL_PADDING),
            &preview.world,
            bounds,
            palette,
        ));
    }
}

fn paint_still_preview(
    painter: &Painter,
    thumbnail: Rect,
    preview: &mut Preview,
    theme: Theme,
    still_drawing_deadline: &mut Option<DateTime<Utc>>,
) {
    let Some(bounds) = &preview.bounds else {
        return;
    };
    let pixels_per_point = painter.pixels_per_point();
    let pixels = (thumbnail.size() * pixels_per_point).round();
    let size_in_pixels = [pixels.x as usize, pixels.y as usize];
    let finished = preview.still_drawing.as_ref().is_some_and(|drawing| {
        drawing.is_for(size_in_pixels, theme) && drawing.unfinished.is_none()
    });
    if let Some(deadline) = (!finished).then(|| still_drawing_deadline.take()).flatten() {
        if preview
            .still_drawing
            .as_ref()
            .is_none_or(|drawing| !drawing.is_for(size_in_pixels, theme))
        {
            let area = Rect::from_min_size(Pos2::ZERO, thumbnail.size()).shrink(THUMBNAIL_PADDING);
            let unfinished = UnfinishedDrawing {
                mesh_in_pixels: tessellate_untextured_in_pixels(
                    top_down_shapes(area, &preview.world, bounds, &theme.palette()),
                    pixels_per_point,
                ),
                next_triangle: 0,
                premultiplied_pixels: vec![[0.0; 4]; size_in_pixels[0] * size_in_pixels[1]],
            };
            let texture = painter.ctx().load_texture(
                "preview",
                unfinished.color_image(size_in_pixels),
                TextureOptions::LINEAR,
            );
            preview.still_drawing = Some(StillDrawing {
                size_in_pixels,
                theme,
                texture,
                unfinished: Some(unfinished),
            });
        }
        if let Some(drawing) = &mut preview.still_drawing {
            if let Some(unfinished) = &mut drawing.unfinished {
                let done = unfinished.fill_triangles_until(drawing.size_in_pixels, deadline);
                drawing.texture.set(
                    unfinished.color_image(drawing.size_in_pixels),
                    TextureOptions::LINEAR,
                );
                if done {
                    drawing.unfinished = None;
                }
            }
        }
    }
    if let Some(drawing) = &preview.still_drawing {
        let whole_texture = Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0));
        painter.image(
            drawing.texture.id(),
            thumbnail,
            whole_texture,
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;
    use crate::examples::EXAMPLE_CATEGORIES;
    use glam::Vec3;

    #[test]
    fn small_examples_run_live_and_large_ones_are_still() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            assert_eq!(preview_mode(&world), PreviewMode::Live, "{}", example.name);
        }
        let mut world = World::new();
        for i in 0..=MAX_LIVE_PREVIEW_ENTITIES {
            Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(i as f32, 0.0, 0.0));
        }
        assert_eq!(preview_mode(&world), PreviewMode::Still);
    }
}
