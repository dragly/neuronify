use std::f32::consts::TAU;

use glam::Vec3;

use crate::components::NeuronType;
use crate::constants::NODE_RADIUS;
use crate::elements::Element;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::Sphere;

pub const GHOST_HEIGHT: f32 = NODE_RADIUS;
const TOUCH_OFFSET_POINTS: f64 = 64.0;
const MOUSE_OFFSET_POINTS: f64 = 20.0;
const TOUCH_LOOSENESS: f32 = 1.0;
const MOUSE_LOOSENESS: f32 = 0.5;
const STIFFNESS: f32 = 120.0;
const DAMPING: f32 = 7.0;
const STEP_SECONDS: f32 = 1.0 / 240.0;
const MAX_FRAME_SECONDS: f32 = 0.1;
const SWAY: f32 = 0.25 * NODE_RADIUS;
const SWAYS_PER_SECOND: f32 = 0.5;
const APPEAR_SECONDS: f32 = 0.15;
const FADE: f32 = 0.55;
const MIST: f32 = 0.5;

pub fn offset_points(touch: bool) -> f64 {
    if touch {
        TOUCH_OFFSET_POINTS
    } else {
        MOUSE_OFFSET_POINTS
    }
}

fn mist(palette: &Palette) -> Vec3 {
    palette.floor.lerp(Vec3::ONE, MIST)
}

#[derive(Clone, Copy, Debug)]
pub struct Ghost {
    element: Element,
    position: Vec3,
    velocity: Vec3,
    looseness: f32,
    age: f32,
}

impl Ghost {
    pub fn new(element: Element, anchor: Vec3) -> Ghost {
        Ghost {
            element,
            position: anchor,
            velocity: Vec3::ZERO,
            looseness: TOUCH_LOOSENESS,
            age: 0.0,
        }
    }

    pub fn follow(&mut self, element: Element, anchor: Vec3, touch: bool, seconds: f32) {
        self.element = element;
        self.looseness = if touch {
            TOUCH_LOOSENESS
        } else {
            MOUSE_LOOSENESS
        };
        self.age += seconds;
        let mut remaining = seconds.min(MAX_FRAME_SECONDS);
        while remaining > 0.0 {
            let step = remaining.min(STEP_SECONDS);
            let pull = STIFFNESS * (anchor - self.position) - DAMPING * self.velocity;
            self.velocity += pull * step;
            self.position += self.velocity * step;
            remaining -= step;
        }
    }

    pub fn sphere(&self, palette: &Palette) -> Option<Sphere> {
        let shown = (self.age / APPEAR_SECONDS).clamp(0.0, 1.0);
        if shown == 0.0 {
            return None;
        }
        let phase = self.age * TAU * SWAYS_PER_SECOND;
        let sway = Vec3::new(phase.sin(), 0.0, (1.3 * phase).sin()) * SWAY * self.looseness;
        let neuron_type = match self.element {
            Element::InhibitoryNeuron => NeuronType::Inhibitory,
            _ => NeuronType::Excitatory,
        };
        Some(Sphere {
            position: self.position + sway,
            color: palette.soma(&neuron_type, 0.0).lerp(mist(palette), FADE),
            radius: NODE_RADIUS * (1.0 - (1.0 - shown).powi(3)),
            _padding: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::colors::Theme;

    const FRAME_SECONDS: f32 = 1.0 / 60.0;

    fn follow_for(ghost: &mut Ghost, seconds: f32, anchor: impl Fn(f32) -> Vec3) {
        let frames = (seconds / FRAME_SECONDS).round() as usize;
        for frame in 1..=frames {
            ghost.follow(
                ghost.element,
                anchor(frame as f32 * FRAME_SECONDS),
                false,
                FRAME_SECONDS,
            );
        }
    }

    #[test]
    fn a_ghost_grows_in_when_painting_starts() {
        let palette = Theme::Light.palette();
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        assert!(ghost.sphere(&palette).is_none());
        follow_for(&mut ghost, 0.05, |_| Vec3::ZERO);
        let radius = ghost.sphere(&palette).unwrap().radius;
        assert!(radius > 0.0 && radius < NODE_RADIUS);
        follow_for(&mut ghost, 0.5, |_| Vec3::ZERO);
        assert_eq!(ghost.sphere(&palette).unwrap().radius, NODE_RADIUS);
    }

    #[test]
    fn a_ghost_hangs_near_a_still_anchor() {
        let palette = Theme::Light.palette();
        let anchor = Vec3::new(3.0, GHOST_HEIGHT, 4.0);
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 3.0, |_| anchor);
        let position = ghost.sphere(&palette).unwrap().position;
        assert!(position.distance(anchor) <= SWAY * MOUSE_LOOSENESS * 1.5);
    }

    #[test]
    fn a_ghost_trails_a_moving_anchor() {
        let speed = 10.0 * NODE_RADIUS;
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 1.0, |time| Vec3::Z * speed * time);
        let lag = speed - ghost.position.z;
        assert!(lag > 0.3 * NODE_RADIUS && lag < NODE_RADIUS, "{lag}");
    }

    #[test]
    fn a_ghost_is_a_faded_neuron() {
        let palette = Theme::Dark.palette();
        let mut ghost = Ghost::new(Element::InhibitoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 0.5, |_| Vec3::ZERO);
        let color = ghost.sphere(&palette).unwrap().color;
        let expected = palette.inhibitory.lerp(mist(&palette), FADE);
        assert!(color.distance(expected) < 1e-6);
    }
}
