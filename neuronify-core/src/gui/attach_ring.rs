use std::f32::consts::PI;

use hecs::Entity;

use super::hold_ring::Look;
use crate::components::Seconds;

const POP: Seconds = Seconds(0.25);
const FADING: Seconds = Seconds(0.15);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttachRing {
    Shown {
        target: Entity,
        since: Seconds,
    },
    Fading {
        target: Entity,
        shown: Seconds,
        since: Seconds,
    },
}

fn swell(age: Seconds) -> f32 {
    let progress = (age / POP).clamp(0.0, 1.0) as f32;
    (PI * progress).sin().max(0.0)
}

impl AttachRing {
    pub fn next(
        ring: Option<AttachRing>,
        ending: Option<Entity>,
        pop: bool,
        now: Seconds,
    ) -> Option<AttachRing> {
        match (ending, ring) {
            (Some(target), Some(AttachRing::Shown { since, .. })) if !pop => {
                Some(AttachRing::Shown { target, since })
            }
            (Some(target), _) => Some(AttachRing::Shown { target, since: now }),
            (None, Some(AttachRing::Shown { target, since })) => Some(AttachRing::Fading {
                target,
                shown: since,
                since: now,
            }),
            (None, ring) => ring.filter(|ring| ring.look(now).is_some()),
        }
    }

    pub fn shown(self) -> Option<Entity> {
        match self {
            AttachRing::Shown { target, .. } => Some(target),
            AttachRing::Fading { .. } => None,
        }
    }

    pub fn target(&self) -> Entity {
        match *self {
            AttachRing::Shown { target, .. } | AttachRing::Fading { target, .. } => target,
        }
    }

    pub fn look(&self, now: Seconds) -> Option<Look> {
        match *self {
            AttachRing::Shown { since, .. } => Some(Look {
                swell: swell(now - since),
                opacity: 1.0,
            }),
            AttachRing::Fading { shown, since, .. } => {
                let opacity = (1.0 - (now - since) / FADING) as f32;
                (opacity > 0.0).then(|| Look {
                    swell: swell(since - shown),
                    opacity,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entities() -> [Entity; 2] {
        let mut world = hecs::World::new();
        [world.spawn(()), world.spawn(())]
    }

    #[test]
    fn entering_pops_the_ring_out_and_back_in_to_rest_on_the_target() {
        let [target, _] = entities();
        let ring = AttachRing::next(None, Some(target), false, Seconds(1.0)).unwrap();
        assert_eq!(ring.look(Seconds(1.0)).unwrap().swell, 0.0);
        assert!((ring.look(Seconds(1.0) + 0.5 * POP).unwrap().swell - 1.0).abs() < 1e-3);
        assert_eq!(ring.look(Seconds(1.0) + 2.0 * POP).unwrap().swell, 0.0);
        assert_eq!(ring.look(Seconds(10.0)).unwrap().opacity, 1.0);
    }

    #[test]
    fn moving_to_a_neighbour_moves_the_ring_without_a_new_pop() {
        let [first, second] = entities();
        let ring = AttachRing::next(None, Some(first), false, Seconds(1.0));
        let moved = AttachRing::next(ring, Some(second), false, Seconds(2.0)).unwrap();
        assert_eq!(moved.target(), second);
        assert_eq!(moved.look(Seconds(2.0)).unwrap().swell, 0.0);
        let popped = AttachRing::next(ring, Some(second), true, Seconds(2.0)).unwrap();
        assert!(popped.look(Seconds(2.0) + 0.5 * POP).unwrap().swell > 0.9);
    }

    #[test]
    fn leaving_fades_the_ring_out() {
        let [target, _] = entities();
        let ring = AttachRing::next(None, Some(target), false, Seconds(1.0));
        let fading = AttachRing::next(ring, None, false, Seconds(2.0));
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
            AttachRing::next(fading, None, false, Seconds(2.01) + FADING),
            None
        );
    }
}
