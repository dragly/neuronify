use std::f32::consts::PI;

use egui::{Context, LayerId, Mesh, Pos2, Shape, Stroke};
use hecs::Entity;

use super::style::Chrome;
use crate::input::HOLD;
use crate::rendering::Theme;

const DELAY: f64 = 0.1;
const GRABBED: f64 = 0.22;
const FADING: f64 = 0.15;
const PEAK_RADIUS: f32 = 64.0;
const MIN_PEAK_GAP: f32 = 24.0;
const RESTING_GAP: f32 = 5.0;
const FILL_OPACITY: f32 = 0.22;
const EDGE_OPACITY: f32 = 0.55;
const EDGE_WIDTH: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoldRing {
    Holding {
        target: Entity,
        held_for: f64,
    },
    Fading {
        target: Entity,
        held_for: f64,
        since: f64,
    },
    Grabbed {
        target: Entity,
        since: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub swell: f32,
    pub opacity: f32,
}

impl Look {
    pub fn gap(&self, thickness: f32) -> f32 {
        let peak = (PEAK_RADIUS - thickness).max(MIN_PEAK_GAP);
        RESTING_GAP + (peak - RESTING_GAP) * self.swell
    }
}

fn swell(held_for: f64) -> f32 {
    let progress = ((held_for - DELAY) / (HOLD - DELAY)).clamp(0.0, 1.0) as f32;
    (PI * progress).sin()
}

impl HoldRing {
    pub fn next(
        ring: Option<HoldRing>,
        pending: Option<(Entity, f64)>,
        grabbed: bool,
        now: f64,
    ) -> Option<HoldRing> {
        match (pending, ring) {
            (Some((target, held_for)), _) => Some(HoldRing::Holding { target, held_for }),
            (None, Some(HoldRing::Holding { target, .. })) if grabbed => {
                Some(HoldRing::Grabbed { target, since: now })
            }
            (None, Some(HoldRing::Holding { target, held_for })) => Some(HoldRing::Fading {
                target,
                held_for,
                since: now,
            }),
            (None, ring) => ring.filter(|ring| ring.look(now).is_some()),
        }
    }

    pub fn target(&self) -> Entity {
        match *self {
            HoldRing::Holding { target, .. }
            | HoldRing::Fading { target, .. }
            | HoldRing::Grabbed { target, .. } => target,
        }
    }

    pub fn look(&self, now: f64) -> Option<Look> {
        let left = |since: f64, duration: f64| (1.0 - (now - since) / duration) as f32;
        match *self {
            HoldRing::Holding { held_for, .. } => (held_for >= DELAY).then(|| Look {
                swell: swell(held_for),
                opacity: 1.0,
            }),
            HoldRing::Fading {
                held_for, since, ..
            } => {
                let opacity = left(since, FADING);
                (held_for >= DELAY && opacity > 0.0).then(|| Look {
                    swell: swell(held_for),
                    opacity,
                })
            }
            HoldRing::Grabbed { since, .. } => {
                let opacity = left(since, GRABBED);
                (opacity > 0.0).then_some(Look {
                    swell: 0.0,
                    opacity,
                })
            }
        }
    }
}

pub fn paint_hold_ring(
    context: &Context,
    theme: Theme,
    inner: &[Pos2],
    outer: &[Pos2],
    look: &Look,
) {
    let accent = Chrome::of(theme).accent;
    let fill = accent.gamma_multiply(FILL_OPACITY * look.opacity);
    let mut band = Mesh::default();
    for (&inside, &outside) in inner.iter().zip(outer) {
        band.colored_vertex(inside, fill);
        band.colored_vertex(outside, fill);
    }
    let corners = 2 * inner.len().min(outer.len()) as u32;
    for corner in (0..corners).step_by(2) {
        let [inside, outside] = [corner, corner + 1];
        let [next_inside, next_outside] = [(corner + 2) % corners, (corner + 3) % corners];
        band.add_triangle(inside, outside, next_outside);
        band.add_triangle(inside, next_outside, next_inside);
    }
    let painter = context.layer_painter(LayerId::background());
    painter.add(band);
    painter.add(Shape::closed_line(
        outer.to_vec(),
        Stroke::new(
            EDGE_WIDTH,
            accent.gamma_multiply(EDGE_OPACITY * look.opacity),
        ),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Entity {
        hecs::World::new().spawn(())
    }

    fn held(held_for: f64) -> Option<HoldRing> {
        HoldRing::next(None, Some((target(), held_for)), false, 0.0)
    }

    #[test]
    fn the_ring_shows_after_a_short_delay_and_swells_out_and_back_in() {
        assert_eq!(held(0.05).unwrap().look(0.0), None);
        let starting = held(DELAY).unwrap().look(0.0).unwrap();
        let halfway = held(0.5 * (DELAY + HOLD)).unwrap().look(0.0).unwrap();
        let held_long_enough = held(HOLD).unwrap().look(0.0).unwrap();
        assert_eq!(starting.gap(0.0), RESTING_GAP);
        assert!((halfway.gap(0.0) - PEAK_RADIUS).abs() < 1e-3);
        assert!((held_long_enough.gap(0.0) - RESTING_GAP).abs() < 1e-3);
    }

    #[test]
    fn a_thick_target_still_gets_a_ring_well_beyond_its_edge() {
        let halfway = held(0.5 * (DELAY + HOLD)).unwrap().look(0.0).unwrap();
        assert!((halfway.gap(100.0) - MIN_PEAK_GAP).abs() < 1e-3);
    }

    #[test]
    fn letting_go_early_fades_the_ring_out() {
        let holding = HoldRing::next(None, Some((target(), 0.2)), false, 1.0);
        let fading = HoldRing::next(holding, None, false, 1.0);
        assert!(matches!(fading, Some(HoldRing::Fading { .. })));
        assert!(fading.unwrap().look(1.1).unwrap().opacity < 1.0);
        assert_eq!(HoldRing::next(fading, None, false, 1.01 + FADING), None);
    }

    #[test]
    fn a_grab_fades_the_ring_on_the_outline() {
        let holding = HoldRing::next(None, Some((target(), HOLD)), false, 1.0);
        let grabbed = HoldRing::next(holding, None, true, 1.0).unwrap();
        let halfway = grabbed.look(1.0 + 0.5 * GRABBED).unwrap();
        assert_eq!(halfway.gap(0.0), RESTING_GAP);
        assert!(halfway.opacity < 1.0);
        assert_eq!(
            HoldRing::next(Some(grabbed), None, false, 1.01 + GRABBED),
            None
        );
    }
}
