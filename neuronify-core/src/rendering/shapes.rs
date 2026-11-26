use glam::Vec3;
use hecs::Entity;

use crate::components::*;
use crate::constants::*;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::CylinderData;
use crate::rendering::scene::Scene;
use crate::stimuli::GeneratorDynamics;
use neuronify_engine::SPIKE_DURATION;

/// How long a soma or generator keeps glowing after it fires.
pub const FLASH_DURATION: Milliseconds = Milliseconds(4.0);
/// Up and right on the screen with the default camera.
pub const SCREEN_UP: Vec3 = Vec3::X;
pub const SCREEN_RIGHT: Vec3 = Vec3::Z;
pub const PAD_RADIUS: f32 = 1.25 * NODE_RADIUS;
pub const GENERATOR_RING_RADIUS: f32 = 1.6 * NODE_RADIUS;
pub const GENERATOR_RING_THICKNESS: f32 = 0.15 * NODE_RADIUS;
const PAD_HALF_HEIGHT: f32 = 0.2;
pub const GLYPH_HEIGHT: f32 = PAD_HALF_HEIGHT + 0.04;

pub const WAVE_STEPS: usize = 16;

/// How much brighter than white a flash starts, so that only flashes bloom.
pub const FLASH_BRIGHTNESS: f32 = 3.0;

pub fn flash(time_since_fire: Milliseconds) -> f32 {
    let remaining = (1.0 - time_since_fire / FLASH_DURATION).max(0.0) as f32;
    remaining * remaining
}

pub fn voltage_fraction(neuron: &LeakyNeuron, state: &NodeState, display: &NeuronDisplay) -> f32 {
    if !display.voltage {
        return 0.0;
    }
    ((state.voltage - neuron.resting_potential) / (neuron.threshold - neuron.resting_potential))
        .clamp(0.0, 1.0) as f32
}

pub fn shown_voltage(state: &NodeState, display: &NeuronDisplay) -> Millivolts {
    if display.voltage {
        state.voltage
    } else {
        RESTING_POTENTIAL
    }
}

/// The colour of a soma, from its voltage and how recently it fired.
pub fn soma_color(
    world: &hecs::World,
    entity: Entity,
    neuron: &LeakyNeuron,
    state: &NodeState,
    display: &NeuronDisplay,
    palette: &Palette,
) -> Vec3 {
    let neuron_type = neuron_type(world, entity);
    let rest = neuron.resting_potential;
    let spiking = state.since_spike < SPIKE_DURATION;
    let color = if display.voltage && state.voltage < rest && !spiking {
        let depth = ((rest - state.voltage) / (rest - neuron.minimum_voltage)).clamp(0.0, 1.0);
        palette.soma_below_rest(&neuron_type, depth as f32)
    } else {
        palette.soma(&neuron_type, voltage_fraction(neuron, state, display))
    };
    color.lerp(palette.flash * FLASH_BRIGHTNESS, flash(state.since_spike))
}

pub fn generator_color(
    dynamics: &GeneratorDynamics,
    neuron_type: NeuronType,
    palette: &Palette,
) -> Vec3 {
    palette.generator(neuron_type).lerp(
        palette.flash * FLASH_BRIGHTNESS,
        flash(dynamics.time_since_fire),
    )
}

pub fn glyph_color() -> Vec3 {
    srgb(255, 244, 214)
}

pub fn pad(scene: &mut Scene, center: Vec3, color: Vec3) {
    scene.cylinders.push(CylinderData {
        start: center - Vec3::Y * PAD_HALF_HEIGHT,
        start_radius: PAD_RADIUS,
        end: center + Vec3::Y * PAD_HALF_HEIGHT,
        end_radius: PAD_RADIUS,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    });
}

pub fn speaker_outline() -> Vec<(f32, f32)> {
    vec![
        (-0.55, -0.2),
        (-0.3, -0.2),
        (0.0, -0.5),
        (0.0, 0.5),
        (-0.3, 0.2),
        (-0.55, 0.2),
    ]
}

pub fn speaker_waves() -> Vec<Vec<(f32, f32)>> {
    [0.25, 0.48]
        .map(|radius| {
            (0..=WAVE_STEPS / 2)
                .map(|step| {
                    let angle = (step as f32 / (WAVE_STEPS / 2) as f32 - 0.5) * 1.6;
                    (0.08 + radius * angle.cos(), radius * angle.sin())
                })
                .collect()
        })
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;

    #[test]
    fn a_soma_below_rest_turns_towards_its_own_colder_colour() {
        let mut world = hecs::World::new();
        let palette = Theme::Dark.palette();
        for (element, below) in [
            (Element::ExcitatoryNeuron, palette.excitatory_below),
            (Element::InhibitoryNeuron, palette.inhibitory_below),
        ] {
            let entity = element.spawn(&mut world, Vec3::ZERO);
            let neuron = world.get::<&LeakyNeuron>(entity).unwrap().clone();
            let color_at = |voltage: Millivolts| {
                let state = NodeState::at(voltage);
                soma_color(
                    &world,
                    entity,
                    &neuron,
                    &state,
                    &NeuronDisplay::default(),
                    &palette,
                )
            };
            let rest = color_at(neuron.resting_potential);
            let halfway = color_at((neuron.resting_potential + neuron.minimum_voltage) / 2.0);
            let floor = color_at(neuron.minimum_voltage);
            assert!(floor.abs_diff_eq(below, 1e-6));
            assert!(halfway.distance(below) < rest.distance(below));
            assert!(halfway.abs_diff_eq(rest.lerp(below, 0.5), 1e-6));
        }
    }

    #[test]
    fn a_soma_keeps_its_resting_colour_when_the_voltage_is_hidden() {
        let mut world = hecs::World::new();
        let entity = world.spawn((NeuronType::Excitatory,));
        let neuron = LeakyNeuron::default();
        let state = NodeState::at(neuron.threshold);
        let palette = Theme::Dark.palette();
        let color = |voltage: bool| {
            let display = NeuronDisplay {
                voltage,
                ..Default::default()
            };
            soma_color(&world, entity, &neuron, &state, &display, &palette)
        };
        assert_eq!(color(false), palette.excitatory);
        assert_ne!(color(true), palette.excitatory);
    }

    #[test]
    fn a_neuron_firing_at_80_hz_shows_its_own_colour_most_of_the_time() {
        let mut world = hecs::World::new();
        let entity = world.spawn((NeuronType::Inhibitory,));
        let neuron = LeakyNeuron::default();
        let palette = Theme::Dark.palette();
        let steps = (Milliseconds(12.5) / DT).round() as usize;
        let unlit = (0..steps)
            .filter(|&step| {
                let state = NodeState {
                    since_spike: step as f64 * DT,
                    ..NodeState::at(neuron.resting_potential)
                };
                let display = NeuronDisplay::default();
                soma_color(&world, entity, &neuron, &state, &display, &palette)
                    == palette.inhibitory
            })
            .count();
        assert!(unlit > steps / 2, "{unlit} of {steps}");
    }
}
