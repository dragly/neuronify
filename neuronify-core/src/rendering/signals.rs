use std::collections::BTreeMap;

use glam::Vec3;
use hecs::{Entity, EntityRef, World};

use crate::components::*;
use crate::constants::RESTING_POTENTIAL;
use crate::rendering::colors::{lit, Palette};
use crate::rendering::gpu_types::{CylinderData, Sphere};
use crate::rendering::paths::ConnectionPath;
use crate::rendering::scene::Scene;
use crate::rendering::transmitter::crossing_time;

const ACTION_POTENTIAL: [(Milliseconds, Millivolts); 31] = [
    (Milliseconds(-1.0), Millivolts(0.1)),
    (Milliseconds(-0.9), Millivolts(0.3)),
    (Milliseconds(-0.8), Millivolts(0.5)),
    (Milliseconds(-0.7), Millivolts(1.1)),
    (Milliseconds(-0.6), Millivolts(2.2)),
    (Milliseconds(-0.5), Millivolts(4.8)),
    (Milliseconds(-0.4), Millivolts(10.4)),
    (Milliseconds(-0.3), Millivolts(22.2)),
    (Milliseconds(-0.2), Millivolts(45.9)),
    (Milliseconds(-0.1), Millivolts(80.0)),
    (Milliseconds(0.0), Millivolts(92.5)),
    (Milliseconds(0.1), Millivolts(87.1)),
    (Milliseconds(0.2), Millivolts(74.7)),
    (Milliseconds(0.3), Millivolts(60.7)),
    (Milliseconds(0.4), Millivolts(47.2)),
    (Milliseconds(0.5), Millivolts(34.8)),
    (Milliseconds(0.6), Millivolts(22.9)),
    (Milliseconds(0.7), Millivolts(11.4)),
    (Milliseconds(0.8), Millivolts(2.1)),
    (Milliseconds(0.9), Millivolts(-3.5)),
    (Milliseconds(1.0), Millivolts(-6.5)),
    (Milliseconds(1.1), Millivolts(-8.2)),
    (Milliseconds(1.2), Millivolts(-9.1)),
    (Milliseconds(1.3), Millivolts(-9.6)),
    (Milliseconds(1.5), Millivolts(-10.0)),
    (Milliseconds(1.7), Millivolts(-10.0)),
    (Milliseconds(2.0), Millivolts(-9.6)),
    (Milliseconds(3.0), Millivolts(-7.5)),
    (Milliseconds(4.0), Millivolts(-5.0)),
    (Milliseconds(5.0), Millivolts(-2.7)),
    (Milliseconds(6.8), Millivolts(0.0)),
];
const FRONT: Milliseconds = ACTION_POTENTIAL[0].0;
const TAIL: Milliseconds = ACTION_POTENTIAL[ACTION_POTENTIAL.len() - 1].0;
const CREST: Millivolts = ACTION_POTENTIAL[10].1;
const SAME_SPIKE: Milliseconds = Milliseconds(0.5);
const COVER: f32 = 1.06;
const BEAD_SCALE: f32 = 2.2;
const THINNEST_BEAD: f32 = 0.3;

fn depolarization(since_crest: Milliseconds) -> Millivolts {
    if !(FRONT..TAIL).contains(&since_crest) {
        return Millivolts::ZERO;
    }
    let after = ACTION_POTENTIAL
        .iter()
        .position(|&(time, _)| time > since_crest)
        .unwrap_or(ACTION_POTENTIAL.len() - 1);
    let ((t0, v0), (t1, v1)) = (ACTION_POTENTIAL[after - 1], ACTION_POTENTIAL[after]);
    v0 + (v1 - v0) * ((since_crest - t0) / (t1 - t0))
}

struct Signal {
    elapsed: Vec<Milliseconds>,
    travel: Milliseconds,
    neuron: Option<NeuronType>,
}

impl Signal {
    fn of(
        world: &World,
        connection: &Connection,
        synapse: &CurrentSynapse,
        now: Milliseconds,
    ) -> Signal {
        let delay = synapse.delay;
        let travel = delay - crossing_time(synapse);
        let mut elapsed: Vec<Milliseconds> = synapse
            .triggers
            .iter()
            .map(|&trigger| now - (trigger - delay))
            .collect();
        let source = world.entity(connection.from).ok();
        let last = source
            .and_then(last_spike)
            .filter(|&last| last < travel + TAIL);
        if let Some(last) = last {
            if elapsed
                .iter()
                .all(|&known| (known - last).abs() > SAME_SPIKE)
            {
                elapsed.push(last);
            }
        }
        let neuron = source
            .filter(|source| !source.has::<GeneratorDynamics>())
            .and_then(|source| source.get::<&NeuronType>().map(|neuron_type| *neuron_type));
        Signal {
            elapsed,
            travel,
            neuron,
        }
    }

    fn along(&self, elapsed: Milliseconds, since_crest: Milliseconds) -> f32 {
        ((elapsed - since_crest) / self.travel) as f32
    }

    fn depolarization_at(&self, along: f32) -> Millivolts {
        self.elapsed
            .iter()
            .map(|&elapsed| depolarization(elapsed - f64::from(along) * self.travel))
            .sum()
    }

    fn lit_stretch(&self) -> Option<(f32, f32)> {
        if self.travel <= Milliseconds::ZERO {
            let lit = self
                .elapsed
                .iter()
                .any(|elapsed| (FRONT..TAIL).contains(elapsed));
            return lit.then_some((0.0, 1.0));
        }
        let reach = |since_crest: Milliseconds| {
            self.elapsed
                .iter()
                .map(move |&elapsed| self.along(elapsed, since_crest).clamp(0.0, 1.0))
        };
        let start = reach(TAIL).reduce(f32::min)?;
        let end = reach(FRONT).reduce(f32::max)?;
        (end > start).then_some((start, end))
    }

    fn crests(&self) -> impl Iterator<Item = f32> + '_ {
        self.elapsed
            .iter()
            .filter(|&&elapsed| {
                self.travel > Milliseconds::ZERO
                    && (Milliseconds::ZERO..self.travel).contains(&elapsed)
            })
            .map(|&elapsed| self.along(elapsed, Milliseconds::ZERO))
    }

    fn voltage_color(&self, base: Vec3, voltage: Millivolts, palette: &Palette) -> Vec3 {
        match self.neuron {
            Some(neuron_type) => palette.compartment(&neuron_type, voltage),
            None => lit(base, voltage),
        }
    }

    fn color(&self, base: Vec3, change: Millivolts, palette: &Palette) -> Vec3 {
        let shift = self.voltage_color(base, RESTING_POTENTIAL + change, palette)
            - self.voltage_color(base, RESTING_POTENTIAL, palette);
        (base + shift).max(Vec3::ZERO)
    }
}

fn last_spike(source: EntityRef) -> Option<Milliseconds> {
    if source.has::<LeakyNeuron>() {
        return source.get::<&NodeState>().map(|state| state.since_spike);
    }
    source
        .get::<&GeneratorDynamics>()
        .map(|dynamics| dynamics.time_since_fire)
}

fn joints(path: &ConnectionPath) -> Vec<f32> {
    let length = path.length();
    let mut along = 0.0;
    path.points
        .windows(2)
        .map(|pair| {
            along += pair[0].distance(pair[1]);
            along / length
        })
        .collect()
}

fn band(
    path: &ConnectionPath,
    signal: &Signal,
    (start, end): (f32, f32),
    palette: &Palette,
    scene: &mut Scene,
) {
    let mut stops: Vec<f32> = signal
        .elapsed
        .iter()
        .filter(|_| signal.travel > Milliseconds::ZERO)
        .flat_map(|&elapsed| {
            ACTION_POTENTIAL
                .iter()
                .map(move |&(since_crest, _)| signal.along(elapsed, since_crest))
        })
        .chain(joints(path))
        .filter(|along| (start..end).contains(along))
        .chain([start, end])
        .collect();
    stops.sort_by(f32::total_cmp);
    stops.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    let radius = path.radius * COVER;
    let samples: Vec<(Vec3, Vec3)> = stops
        .iter()
        .map(|&along| {
            let base = path.start_color.lerp(path.end_color, along);
            let change = signal.depolarization_at(along);
            (path.point_at(along), signal.color(base, change, palette))
        })
        .collect();
    for pair in samples.windows(2) {
        scene.cylinders.push(CylinderData {
            start: pair[0].0,
            start_radius: radius,
            end: pair[1].0,
            end_radius: radius,
            start_color: pair[0].1,
            end_color: pair[1].1,
            _padding: Default::default(),
        });
    }
    let inner = samples.len().saturating_sub(1);
    scene.spheres.extend(
        samples[..inner]
            .iter()
            .skip(1)
            .map(|&(position, color)| Sphere {
                position,
                color,
                radius,
                _padding: Default::default(),
            }),
    );
}

pub fn collect_signals(
    world: &World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    palette: &Palette,
    now: Milliseconds,
) -> Scene {
    let mut scene = Scene::default();
    for (entity, (connection, synapse)) in world
        .query::<(Entity, (&Connection, &CurrentSynapse))>()
        .iter()
    {
        let Some(path) = paths.get(&entity) else {
            continue;
        };
        let signal = Signal::of(world, connection, synapse, now);
        if let Some(stretch) = signal.lit_stretch() {
            band(path, &signal, stretch, palette, &mut scene);
        }
        scene.spheres.extend(signal.crests().map(|along| {
            let base = path.start_color.lerp(path.end_color, along);
            Sphere {
                position: path.point_at(along),
                color: signal.color(base, CREST, palette),
                radius: (path.radius * BEAD_SCALE).max(THINNEST_BEAD),
                _padding: Default::default(),
            }
        }));
    }
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::NODE_RADIUS;
    use crate::rendering::cylinders::compartment_radii;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::Theme;
    use crate::simulation::model::ENDING_DELAY;

    struct Synapse {
        world: World,
        synapse: Entity,
        source: Entity,
    }

    fn synapse(source_type: NeuronType, triggers: Vec<f64>) -> Synapse {
        let mut world = World::new();
        let mut neuron = |x: f32, neuron_type: NeuronType| {
            world.spawn((
                Position {
                    position: Vec3::new(x, 0.0, 0.0),
                },
                LeakyNeuron::default(),
                NodeState::default(),
                neuron_type,
            ))
        };
        let source = neuron(0.0, source_type);
        let target = neuron(20.0, NeuronType::Excitatory);
        let synapse = world.spawn((
            Connection {
                from: source,
                to: target,
                strength: 1.0,
                directional: true,
            },
            CurrentSynapse {
                delay: Milliseconds(4.0) + ENDING_DELAY,
                triggers: triggers
                    .into_iter()
                    .map(|time| Milliseconds(time) + ENDING_DELAY)
                    .collect(),
                ..Default::default()
            },
        ));
        Synapse {
            world,
            synapse,
            source,
        }
    }

    fn draw(synapse: &Synapse, now: f64) -> (Scene, ConnectionPath) {
        let palette = Theme::Dark.palette();
        let radii = compartment_radii(&synapse.world);
        let mut paths = connection_paths(&synapse.world, &radii, &palette);
        let scene = collect_signals(&synapse.world, &paths, &palette, Milliseconds(now));
        (scene, paths.remove(&synapse.synapse).unwrap())
    }

    fn beads(scene: &Scene, path: &ConnectionPath) -> Vec<Sphere> {
        let radius = (path.radius * BEAD_SCALE).max(THINNEST_BEAD);
        scene
            .spheres
            .iter()
            .filter(|sphere| sphere.radius == radius)
            .copied()
            .collect()
    }

    fn brightest(scene: &Scene) -> Option<(Vec3, f32)> {
        scene
            .cylinders
            .iter()
            .flat_map(|cylinder| {
                [
                    (cylinder.start, cylinder.start_color),
                    (cylinder.end, cylinder.end_color),
                ]
            })
            .map(|(position, color)| (position, color.max_element()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }

    #[test]
    fn a_quiet_synapse_shows_no_signal() {
        let (scene, _) = draw(&synapse(NeuronType::Excitatory, vec![]), 1000.0);
        assert!(scene.spheres.is_empty());
        assert!(scene.cylinders.is_empty());
    }

    #[test]
    fn a_spike_lights_the_synapse_where_its_crest_is() {
        let firing = synapse(NeuronType::Excitatory, vec![1000.0]);
        let (scene, path) = draw(&firing, 997.0);

        let beads = beads(&scene, &path);
        assert_eq!(beads.len(), 1);
        let a_quarter_along = path.point_at(0.25);
        assert!(beads[0].position.distance(a_quarter_along) < 1e-5);
        let (position, brightness) = brightest(&scene).unwrap();
        assert!(position.distance(a_quarter_along) < 0.5 * NODE_RADIUS);
        assert!(brightness > path.start_color.max_element());
    }

    #[test]
    fn a_spike_reaches_the_terminal_in_time_for_its_transmitter_to_cross() {
        let firing = synapse(NeuronType::Excitatory, vec![1000.0]);
        let (scene, path) = draw(&firing, 999.9);
        let arriving = beads(&scene, &path);
        assert_eq!(arriving.len(), 1);
        assert!(arriving[0].position.distance(path.point_at(1.0)) < 0.05 * path.length());

        let (scene, path) = draw(&firing, 1000.1);
        assert!(beads(&scene, &path).is_empty());
    }

    #[test]
    fn a_signal_glares_like_an_axon_of_its_neuron() {
        let palette = Theme::Dark.palette();
        for neuron_type in [NeuronType::Excitatory, NeuronType::Inhibitory] {
            let firing = synapse(neuron_type, vec![1000.0]);
            let (scene, path) = draw(&firing, 997.0);
            let bead = beads(&scene, &path)[0];
            let crest = palette.compartment(&neuron_type, RESTING_POTENTIAL + CREST);
            assert!((bead.color - crest).abs().max_element() < 0.5);
        }
    }

    #[test]
    fn the_tail_stays_after_the_spike_has_arrived() {
        let arrived = synapse(NeuronType::Excitatory, vec![]);
        arrived
            .world
            .get::<&mut NodeState>(arrived.source)
            .unwrap()
            .since_spike = Milliseconds(4.4);
        let (scene, path) = draw(&arrived, 1000.0);

        assert!(beads(&scene, &path).is_empty());
        let (position, brightness) = brightest(&scene).unwrap();
        assert!(position.distance(path.point_at(1.0)) < 0.5 * NODE_RADIUS);
        assert!(brightness > path.end_color.max_element());
    }

    #[test]
    fn the_band_fades_into_the_tube_ahead_of_the_crest() {
        let (scene, path) = draw(&synapse(NeuronType::Excitatory, vec![1000.0]), 997.0);
        let front = scene.cylinders.last().unwrap();
        assert!(front.end.distance(path.point_at(0.5)) < 1e-4);
        assert!(
            front
                .end_color
                .distance(path.start_color.lerp(path.end_color, 0.5))
                < 1e-2
        );
    }
}
