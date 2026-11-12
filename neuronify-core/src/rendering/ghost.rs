use std::f32::consts::TAU;

use glam::Vec3;
use hecs::Entity;

use crate::components::{NeuronType, Seconds};
use crate::constants::NODE_RADIUS;
use crate::elements::Element;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::Sphere;

pub const GHOST_HEIGHT: f32 = NODE_RADIUS;
const TOUCH_OFFSET_POINTS: f64 = 64.0;
const MOUSE_OFFSET_POINTS: f64 = 20.0;
const TOUCH: Feel = Feel {
    looseness: 1.0,
    stiffness: 120.0,
    damping: 7.0,
};
const MOUSE: Feel = Feel {
    looseness: 0.5,
    stiffness: 200.0,
    damping: 16.0,
};
const STEP_SECONDS: f32 = 1.0 / 240.0;
const MAX_FRAME_SECONDS: f32 = 0.1;
const SWAY: f32 = 0.25 * NODE_RADIUS;
const SWAYS_PER_SECOND: f32 = 0.5;
const FADE: f32 = 0.55;
const MIST: f32 = 0.5;
const APPEAR_SECONDS: f32 = 0.15;
pub const LANDING: Seconds = Seconds(0.22);

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
struct Feel {
    looseness: f32,
    stiffness: f32,
    damping: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Ghost {
    element: Element,
    position: Vec3,
    velocity: Vec3,
    feel: Feel,
    age: f32,
    way: f32,
}

impl Ghost {
    pub fn new(element: Element, anchor: Vec3) -> Ghost {
        Ghost {
            element,
            position: anchor,
            velocity: Vec3::ZERO,
            feel: TOUCH,
            age: 0.0,
            way: 0.0,
        }
    }

    pub fn follow(&mut self, element: Element, anchor: Vec3, way: f32, touch: bool, seconds: f32) {
        self.element = element;
        self.way = way;
        self.feel = if touch { TOUCH } else { MOUSE };
        self.age += seconds;
        let mut remaining = seconds.min(MAX_FRAME_SECONDS);
        while remaining > 0.0 {
            let step = remaining.min(STEP_SECONDS);
            let pull =
                self.feel.stiffness * (anchor - self.position) - self.feel.damping * self.velocity;
            self.velocity += pull * step;
            self.position += self.velocity * step;
            remaining -= step;
        }
    }

    pub fn sphere(&self, palette: &Palette) -> Option<Sphere> {
        let shown = (self.age / APPEAR_SECONDS).clamp(0.0, 1.0);
        if shown == 0.0 || self.way <= 0.0 {
            return None;
        }
        let phase = self.age * TAU * SWAYS_PER_SECOND;
        let sway = Vec3::new(phase.sin(), 0.0, (1.3 * phase).sin()) * SWAY * self.feel.looseness;
        let neuron_type = match self.element {
            Element::InhibitoryNeuron => NeuronType::Inhibitory,
            _ => NeuronType::Excitatory,
        };
        Some(Sphere {
            position: self.position + sway,
            color: palette.soma(&neuron_type, 0.0).lerp(mist(palette), FADE),
            radius: NODE_RADIUS * (1.0 - (1.0 - shown).powi(3)) * self.way,
            _padding: Default::default(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LandingGhost {
    pub neuron: Entity,
    from: Sphere,
    since: Seconds,
}

impl LandingGhost {
    pub fn new(neuron: Entity, from: Sphere, now: Seconds) -> LandingGhost {
        LandingGhost {
            neuron,
            from,
            since: now,
        }
    }

    pub fn progress(&self, now: Seconds) -> f32 {
        ((now - self.since) / LANDING).clamp(0.0, 1.0) as f32
    }

    pub fn sphere(&self, soma: Sphere, now: Seconds) -> Sphere {
        let fall = self.progress(now).powi(2);
        Sphere {
            position: self.from.position.lerp(soma.position, fall),
            color: self.from.color.lerp(soma.color, fall),
            radius: self.from.radius + (soma.radius - self.from.radius) * fall,
            _padding: Default::default(),
        }
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
                1.0,
                false,
                FRAME_SECONDS,
            );
        }
    }

    #[test]
    fn a_ghost_grows_in_when_it_appears() {
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
    fn a_ghost_grows_with_the_way_to_the_next_neuron() {
        let palette = Theme::Light.palette();
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 0.5, |_| Vec3::ZERO);
        let mut radius_at = |way| {
            ghost.follow(ghost.element, Vec3::ZERO, way, true, FRAME_SECONDS);
            ghost.sphere(&palette).map(|sphere| sphere.radius)
        };
        assert_eq!(radius_at(0.0), None);
        assert_eq!(radius_at(0.25), Some(0.25 * NODE_RADIUS));
        assert_eq!(radius_at(1.0), Some(NODE_RADIUS));
    }

    #[test]
    fn a_landing_ghost_falls_onto_the_soma_and_takes_its_look() {
        let palette = Theme::Light.palette();
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        let above = Vec3::new(0.0, GHOST_HEIGHT, -3.0);
        follow_for(&mut ghost, 3.0, |_| above);
        let from = ghost.sphere(&palette).unwrap();
        let soma = Sphere {
            position: Vec3::ZERO,
            color: palette.excitatory,
            radius: 1.2 * NODE_RADIUS,
            _padding: Default::default(),
        };
        let landing = LandingGhost::new(Entity::DANGLING, from, Seconds(10.0));
        let start = landing.sphere(soma, Seconds(10.0));
        assert_eq!((start.position, start.radius), (from.position, from.radius));
        let halfway = landing.sphere(soma, Seconds(10.0) + LANDING * 0.5);
        let fallen = from.position.distance(halfway.position);
        assert!(fallen > 0.0 && fallen < 0.5 * from.position.distance(soma.position));
        let landed = landing.sphere(soma, Seconds(10.0) + LANDING);
        assert_eq!(landed.position, soma.position);
        assert_eq!(landed.radius, soma.radius);
        assert!(landed.color.distance(soma.color) < 1e-6);
        assert_eq!(landing.progress(Seconds(10.0) + LANDING * 2.0), 1.0);
    }

    #[test]
    fn a_ghost_hangs_near_a_still_anchor() {
        let palette = Theme::Light.palette();
        let anchor = Vec3::new(3.0, GHOST_HEIGHT, 4.0);
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 3.0, |_| anchor);
        let position = ghost.sphere(&palette).unwrap().position;
        assert!(position.distance(anchor) <= SWAY * MOUSE.looseness * 1.5);
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
    fn a_ghost_barely_swings_past_a_mouse_that_stops() {
        let speed = 10.0 * NODE_RADIUS;
        let stop = 0.5 * speed;
        let mut ghost = Ghost::new(Element::ExcitatoryNeuron, Vec3::ZERO);
        follow_for(&mut ghost, 0.5, |time| Vec3::Z * speed * time);
        let mut farthest: f32 = 0.0;
        for _ in 0..60 {
            follow_for(&mut ghost, FRAME_SECONDS, |_| Vec3::Z * stop);
            farthest = farthest.max(ghost.position.z - stop);
        }
        assert!(farthest < 0.25 * NODE_RADIUS, "{farthest}");
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
