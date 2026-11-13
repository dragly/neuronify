use std::collections::HashMap;

use hecs::{Entity, World};

use crate::components::{
    Compartment, Connection, CurrentClamp, GeneratorDynamics, LeakyNeuron, NodeState, Position,
    Seconds,
};
use crate::tools::Tool;

const LATE: Seconds = Seconds(8.0);
const ANSWER: Seconds = Seconds(3.0);
const FADE: Seconds = Seconds(0.25);

#[derive(Default)]
pub struct FireHint {
    learned: bool,
    quiet: bool,
    waited: bool,
    shape: (usize, usize),
    still_since: Seconds,
    tapped: Option<(Entity, Seconds)>,
}

impl FireHint {
    pub fn forget_world(&mut self) {
        self.quiet = false;
        self.waited = false;
        self.tapped = None;
    }

    pub fn update(&mut self, world: &World, now: Seconds) {
        if self.learned {
            return;
        }
        let shape = (
            world.query::<&Position>().iter().count(),
            world.query::<&Connection>().iter().count(),
        );
        if shape != self.shape {
            self.shape = shape;
            self.still_since = now;
        }
        self.quiet = quiet(world);
        if self.quiet {
            self.waited = true;
        } else if self.waited && fired(world) {
            self.learned = true;
            self.tapped = None;
        }
    }

    pub fn tap(&mut self, world: &World, tapped: Entity, tool: Tool, now: Seconds) -> bool {
        if !self.quiet || tool == Tool::Stimulate || !somas(world).contains_key(&tapped) {
            return false;
        }
        self.tapped = Some((tapped, now));
        true
    }

    pub fn answer(&self, world: &World, now: Seconds) -> Option<(Entity, f32)> {
        let (tapped, _) = self.tapped?;
        let age = self.answer_age(now)?;
        if !world.contains(tapped) {
            return None;
        }
        let shown = (age / FADE).min((ANSWER - age) / FADE).clamp(0.0, 1.0);
        Some((tapped, shown as f32))
    }

    pub fn pulse(&self, tool: Tool, now: Seconds) -> Option<Seconds> {
        if !self.quiet || tool == Tool::Stimulate {
            return None;
        }
        let late = now - self.still_since - LATE;
        self.answer_age(now)
            .or((late >= Seconds::ZERO).then_some(late))
    }

    fn answer_age(&self, now: Seconds) -> Option<Seconds> {
        let (_, since) = self.tapped?;
        let age = now - since;
        (self.quiet && age >= Seconds::ZERO && age < ANSWER).then_some(age)
    }
}

fn quiet(world: &World) -> bool {
    let inputs = world
        .query::<()>()
        .with::<hecs::Or<&CurrentClamp, &GeneratorDynamics>>()
        .iter()
        .next()
        .is_some();
    !inputs && !fired(world) && linked(world)
}

fn fired(world: &World) -> bool {
    world
        .query::<&NodeState>()
        .with::<&LeakyNeuron>()
        .iter()
        .any(|state| state.since_spike.0.is_finite())
}

fn linked(world: &World) -> bool {
    let somas = somas(world);
    world
        .query::<&Connection>()
        .iter()
        .filter(|connection| connection.directional)
        .any(
            |connection| match (somas.get(&connection.from), somas.get(&connection.to)) {
                (Some(from), Some(to)) => from != to,
                _ => false,
            },
        )
}

fn somas(world: &World) -> HashMap<Entity, Entity> {
    let mut branches: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for connection in world.query::<&Connection>().iter() {
        if connection.directional {
            continue;
        }
        branches
            .entry(connection.from)
            .or_default()
            .push(connection.to);
        branches
            .entry(connection.to)
            .or_default()
            .push(connection.from);
    }
    let mut somas = HashMap::new();
    for (soma, _) in world.query::<(Entity, &LeakyNeuron)>().iter() {
        let mut frontier = vec![soma];
        while let Some(node) = frontier.pop() {
            if somas.contains_key(&node) {
                continue;
            }
            somas.insert(node, soma);
            frontier.extend(
                branches
                    .get(&node)
                    .into_iter()
                    .flatten()
                    .filter(|&&next| world.satisfies::<&Compartment>(next)),
            );
        }
    }
    somas
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;
    use crate::components::{ConductanceSynapse, Dendrite, Milliseconds};
    use crate::dendrites::spawn_dendrites;
    use crate::editor::Editor;
    use crate::elements::{spawn_synapse, Element};

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    fn neuron(world: &mut World, position: Vec3) -> Entity {
        Element::ExcitatoryNeuron.spawn(world, position)
    }

    fn connected_pair() -> (World, Entity, Entity) {
        let mut world = World::new();
        let a = neuron(&mut world, at(0.0, 0.0));
        let b = neuron(&mut world, at(10.0, 0.0));
        spawn_synapse(&mut world, a, b);
        (world, a, b)
    }

    fn pulse_after(world: &World, seconds: f64) -> Option<Seconds> {
        let mut hint = FireHint::default();
        hint.update(world, Seconds::ZERO);
        hint.update(world, Seconds(seconds));
        hint.pulse(Tool::View, Seconds(seconds))
    }

    fn fire(world: &mut World, neuron: Entity) {
        world.get::<&mut NodeState>(neuron).unwrap().since_spike = Milliseconds(1.0);
    }

    fn axon_between_compartmental_neurons() -> (World, Entity, Entity) {
        let mut world = World::new();
        let source = neuron(&mut world, at(0.0, 0.0));
        let target = neuron(&mut world, at(0.0, 30.0));
        spawn_dendrites(&mut world, source);
        spawn_dendrites(&mut world, target);
        let mut editor = Editor::default();
        editor.tool = Tool::Axon;
        for step in 0..=80 {
            if step > 0 {
                editor.move_held();
            }
            editor.edit(&mut world, at(0.0, 30.0 * step as f32 / 80.0));
        }
        editor.lift(&mut world);
        (world, source, target)
    }

    #[test]
    fn a_quiet_circuit_pulses_stimulate_after_eight_still_seconds() {
        let (world, _, _) = connected_pair();
        assert_eq!(pulse_after(&world, 7.9), None);
        assert!(pulse_after(&world, 8.1).is_some());

        let mut hint = FireHint::default();
        hint.update(&world, Seconds::ZERO);
        hint.update(&world, Seconds(9.0));
        assert_eq!(hint.pulse(Tool::Stimulate, Seconds(9.0)), None);
    }

    #[test]
    fn editing_starts_the_wait_over() {
        let (mut world, _, _) = connected_pair();
        let mut hint = FireHint::default();
        hint.update(&world, Seconds::ZERO);
        neuron(&mut world, at(20.0, 0.0));
        hint.update(&world, Seconds(5.0));
        hint.update(&world, Seconds(12.0));
        assert_eq!(hint.pulse(Tool::View, Seconds(12.0)), None);
        hint.update(&world, Seconds(13.5));
        assert!(hint.pulse(Tool::View, Seconds(13.5)).is_some());
    }

    #[test]
    fn neurons_without_a_connection_or_with_an_input_get_no_hint() {
        let mut lone = World::new();
        neuron(&mut lone, at(0.0, 0.0));
        neuron(&mut lone, at(10.0, 0.0));
        assert_eq!(pulse_after(&lone, 20.0), None);

        let (mut fed, _, _) = connected_pair();
        Element::TouchSensor.spawn(&mut fed, at(-10.0, 0.0));
        assert_eq!(pulse_after(&fed, 20.0), None);
    }

    #[test]
    fn tapping_a_quiet_neuron_answers_at_once_for_three_seconds() {
        let (world, _, b) = connected_pair();
        let mut hint = FireHint::default();
        hint.update(&world, Seconds::ZERO);
        assert!(!hint.tap(&world, b, Tool::Stimulate, Seconds(1.0)));
        assert!(hint.tap(&world, b, Tool::View, Seconds(1.0)));
        assert_eq!(hint.answer(&world, Seconds(2.0)).map(|(at, _)| at), Some(b));
        assert!(hint.pulse(Tool::View, Seconds(2.0)).is_some());
        assert_eq!(hint.answer(&world, Seconds(4.1)), None);
        assert_eq!(hint.pulse(Tool::View, Seconds(4.1)), None);
    }

    #[test]
    fn once_a_neuron_fired_the_hint_stays_away_for_the_session() {
        let (mut world, a, b) = connected_pair();
        let mut hint = FireHint::default();
        hint.update(&world, Seconds::ZERO);
        fire(&mut world, a);
        hint.update(&world, Seconds(1.0));
        hint.update(&world, Seconds(20.0));
        assert_eq!(hint.pulse(Tool::View, Seconds(20.0)), None);
        assert!(!hint.tap(&world, b, Tool::View, Seconds(20.0)));

        hint.forget_world();
        let (fresh, _, b) = connected_pair();
        hint.update(&fresh, Seconds(30.0));
        hint.update(&fresh, Seconds(50.0));
        assert_eq!(hint.pulse(Tool::View, Seconds(50.0)), None);
        assert!(!hint.tap(&fresh, b, Tool::View, Seconds(50.0)));
    }

    #[test]
    fn an_axon_between_compartmental_neurons_counts_as_a_circuit() {
        let (world, source, target) = axon_between_compartmental_neurons();
        assert!(world.query::<&ConductanceSynapse>().iter().next().is_some());
        assert!(pulse_after(&world, 8.1).is_some());

        let somas = somas(&world);
        let axon = world
            .query::<(Entity, &Compartment)>()
            .without::<&Dendrite>()
            .iter()
            .map(|(compartment, _)| compartment)
            .find(|compartment| somas.get(compartment) == Some(&source))
            .unwrap();
        let mut hint = FireHint::default();
        hint.update(&world, Seconds::ZERO);
        assert!(hint.tap(&world, axon, Tool::View, Seconds(1.0)));
        assert!(hint.tap(&world, target, Tool::View, Seconds(1.0)));
    }

    #[test]
    fn compartmental_neurons_with_dendrites_alone_get_no_hint() {
        let mut world = World::new();
        for x in [0.0, 30.0] {
            let soma = neuron(&mut world, at(x, 0.0));
            spawn_dendrites(&mut world, soma);
        }
        assert_eq!(pulse_after(&world, 20.0), None);
    }
}
