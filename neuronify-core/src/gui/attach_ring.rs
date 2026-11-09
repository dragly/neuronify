use std::f32::consts::PI;

use hecs::Entity;

use super::hold_ring::Look;
use crate::components::Seconds;

const POP: Seconds = Seconds(0.25);
const FADING: Seconds = Seconds(0.15);
const CLOSING_DELAY: Seconds = Seconds(0.1);
const OPEN: f32 = 0.6;
const BURST: Seconds = Seconds(0.4);
const BURST_SWELL: f32 = 0.45;

pub const WAITING: Look = Look {
    swell: 0.0,
    opacity: 0.5,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttachRing {
    Shown {
        target: Entity,
        since: Seconds,
    },
    Closing {
        target: Entity,
        progress: f32,
    },
    Fading {
        target: Entity,
        look: Look,
        since: Seconds,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachBurst {
    pub target: Entity,
    pub since: Seconds,
}

fn swell(age: Seconds) -> f32 {
    let progress = (age / POP).clamp(0.0, 1.0) as f32;
    (PI * progress).sin().max(0.0)
}

pub fn dwell_progress(held_for: Seconds, dwell: Seconds) -> f32 {
    ((held_for - CLOSING_DELAY) / (dwell - CLOSING_DELAY)).clamp(0.0, 1.0) as f32
}

fn closing(progress: f32) -> f32 {
    OPEN * (1.0 - progress * progress * (3.0 - 2.0 * progress))
}

impl AttachRing {
    pub fn next(
        ring: Option<AttachRing>,
        ending: Option<Entity>,
        progress: Option<f32>,
        pop: bool,
        now: Seconds,
    ) -> Option<AttachRing> {
        match (ending, progress, ring) {
            (Some(target), Some(progress), _) => Some(AttachRing::Closing { target, progress }),
            (Some(target), None, Some(AttachRing::Shown { since, .. })) if !pop => {
                Some(AttachRing::Shown { target, since })
            }
            (Some(target), None, _) => Some(AttachRing::Shown { target, since: now }),
            (None, _, Some(ring)) if ring.shown().is_some() => Some(AttachRing::Fading {
                target: ring.target(),
                look: ring.look(now)?,
                since: now,
            }),
            (None, _, ring) => ring.filter(|ring| ring.look(now).is_some()),
        }
    }

    pub fn shown(self) -> Option<Entity> {
        match self {
            AttachRing::Shown { target, .. } | AttachRing::Closing { target, .. } => Some(target),
            AttachRing::Fading { .. } => None,
        }
    }

    pub fn target(&self) -> Entity {
        match *self {
            AttachRing::Shown { target, .. }
            | AttachRing::Closing { target, .. }
            | AttachRing::Fading { target, .. } => target,
        }
    }

    pub fn look(&self, now: Seconds) -> Option<Look> {
        match *self {
            AttachRing::Shown { since, .. } => Some(Look {
                swell: swell(now - since),
                opacity: 1.0,
            }),
            AttachRing::Closing { progress, .. } => Some(Look {
                swell: closing(progress),
                opacity: 1.0,
            }),
            AttachRing::Fading { look, since, .. } => {
                let opacity = look.opacity * (1.0 - (now - since) / FADING) as f32;
                (opacity > 0.0).then_some(Look { opacity, ..look })
            }
        }
    }
}

impl AttachBurst {
    pub fn look(&self, now: Seconds) -> Option<Look> {
        let progress = ((now - self.since) / BURST) as f32;
        (0.0..1.0).contains(&progress).then(|| Look {
            swell: BURST_SWELL * (1.0 - (1.0 - progress).powi(3)),
            opacity: 1.0 - progress * progress,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::DWELL;

    fn entities() -> [Entity; 2] {
        let mut world = hecs::World::new();
        [world.spawn(()), world.spawn(())]
    }

    #[test]
    fn entering_pops_the_ring_out_and_back_in_to_rest_on_the_target() {
        let [target, _] = entities();
        let ring = AttachRing::next(None, Some(target), None, false, Seconds(1.0)).unwrap();
        assert_eq!(ring.look(Seconds(1.0)).unwrap().swell, 0.0);
        assert!((ring.look(Seconds(1.0) + 0.5 * POP).unwrap().swell - 1.0).abs() < 1e-3);
        assert_eq!(ring.look(Seconds(1.0) + 2.0 * POP).unwrap().swell, 0.0);
        assert_eq!(ring.look(Seconds(10.0)).unwrap().opacity, 1.0);
    }

    #[test]
    fn moving_to_a_neighbour_moves_the_ring_without_a_new_pop() {
        let [first, second] = entities();
        let ring = AttachRing::next(None, Some(first), None, false, Seconds(1.0));
        let moved = AttachRing::next(ring, Some(second), None, false, Seconds(2.0)).unwrap();
        assert_eq!(moved.target(), second);
        assert_eq!(moved.look(Seconds(2.0)).unwrap().swell, 0.0);
        let popped = AttachRing::next(ring, Some(second), None, true, Seconds(2.0)).unwrap();
        assert!(popped.look(Seconds(2.0) + 0.5 * POP).unwrap().swell > 0.9);
    }

    #[test]
    fn leaving_fades_the_ring_out() {
        let [target, _] = entities();
        let ring = AttachRing::next(None, Some(target), None, false, Seconds(1.0));
        let fading = AttachRing::next(ring, None, None, false, Seconds(2.0));
        assert_eq!(fading.unwrap().shown(), None);
        assert!(
            fading
                .unwrap()
                .look(Seconds(2.0) + 0.5 * FADING)
                .unwrap()
                .opacity
                < 1.0
        );
        assert_eq!(
            AttachRing::next(fading, None, None, false, Seconds(2.01) + FADING),
            None
        );
    }

    #[test]
    fn holding_still_closes_the_ring_from_wide_open_onto_the_target() {
        let [target, _] = entities();
        let held = |held_for: Seconds, dwell: Seconds| {
            let progress = dwell_progress(held_for, dwell);
            let ring = AttachRing::next(None, Some(target), Some(progress), true, Seconds(1.0));
            ring.unwrap().look(Seconds(1.0)).unwrap().swell
        };
        assert_eq!(held(Seconds(0.0), DWELL), OPEN);
        assert_eq!(held(CLOSING_DELAY, DWELL), OPEN);
        let halfway = held(0.5 * (CLOSING_DELAY + DWELL), DWELL);
        assert!(0.0 < halfway && halfway < OPEN);
        assert_eq!(held(DWELL, DWELL), 0.0);
        assert!(held(DWELL, 2.0 * DWELL) > 0.0);
        assert_eq!(held(2.0 * DWELL, 2.0 * DWELL), 0.0);
    }

    #[test]
    fn leaving_a_closing_ring_fades_it_where_it_was() {
        let [target, _] = entities();
        let ring = AttachRing::next(None, Some(target), Some(0.0), false, Seconds(1.0));
        let fading = AttachRing::next(ring, None, None, false, Seconds(1.0)).unwrap();
        let look = fading.look(Seconds(1.0) + 0.5 * FADING).unwrap();
        assert_eq!(look.swell, OPEN);
        assert!(look.opacity < 1.0);
    }

    #[test]
    fn an_attach_bursts_out_from_the_target_and_fades() {
        let [target, _] = entities();
        let burst = AttachBurst {
            target,
            since: Seconds(1.0),
        };
        let start = burst.look(Seconds(1.0)).unwrap();
        let later = burst.look(Seconds(1.0) + 0.5 * BURST).unwrap();
        assert_eq!((start.swell, start.opacity), (0.0, 1.0));
        assert!(later.swell > start.swell && later.opacity < start.opacity);
        assert_eq!(burst.look(Seconds(1.0) + BURST), None);
    }
}
