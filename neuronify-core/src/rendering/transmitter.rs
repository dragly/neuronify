use std::collections::{BTreeMap, HashMap};
use std::f32::consts::TAU;

use glam::Vec3;
use hecs::{Entity, World};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::components::*;
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::{HaloData, Sphere};
use crate::rendering::paths::{full_terminal_radius, ConnectionPath};
use crate::rendering::scene::Scene;
use crate::rendering::spheres::soma_radius;
use crate::simulation::model::{ending_site_scale, ENDING_DELAY};

const FAINTEST_BOUND: f32 = 0.05;
const PARTICLES: usize = 10;
const PARTICLE_PER_TERMINAL_RADIUS: f32 = 0.35;
const WIDEST_LEAVING_ANGLE: f32 = 1.0;
const WIDEST_LANDING_ANGLE: f32 = 1.2;
const FASTEST_CROSSING: f32 = 0.45;
const HALO_PER_PARTICLE_RADIUS: f32 = 4.0;
const GOLDEN_FRACTION: f32 = 0.618_034;

pub fn crossing_time(synapse: &CurrentSynapse) -> Milliseconds {
    synapse.delay.clamp(Milliseconds::ZERO, ENDING_DELAY)
}

struct Release {
    bound: f32,
    pending: Vec<Milliseconds>,
    crossing: Milliseconds,
    particles: usize,
}

impl Release {
    fn crossings(&self, now: Milliseconds) -> Vec<(Milliseconds, f32)> {
        self.pending
            .iter()
            .map(|&opening| (opening, 1.0 - ((opening - now) / self.crossing) as f32))
            .filter(|&(_, through)| through >= 0.0)
            .collect()
    }
}

#[derive(Default)]
pub struct Landings(HashMap<Entity, Landing>);

#[derive(Default)]
struct Landing {
    opened: Option<Milliseconds>,
    pending: Vec<Milliseconds>,
}

impl Landings {
    fn last_opened(&mut self, synapse: Entity, pending: &[Milliseconds]) -> Option<Milliseconds> {
        if pending.is_empty() && !self.0.contains_key(&synapse) {
            return None;
        }
        let landing = self.0.entry(synapse).or_default();
        landing.opened = landing
            .pending
            .iter()
            .copied()
            .filter(|opening| !pending.contains(opening))
            .chain(landing.opened)
            .reduce(Milliseconds::max);
        landing.pending.clear();
        landing.pending.extend_from_slice(pending);
        landing.opened
    }
}

fn release(world: &World, entity: Entity, connection: &Connection) -> Option<Release> {
    if let Ok(synapse) = world.get::<&CurrentSynapse>(entity) {
        let bound = match synapse.alpha_function {
            true => synapse.exponential / std::f64::consts::E,
            false => synapse.exponential,
        };
        return Some(Release {
            bound: bound.clamp(0.0, 1.0) as f32,
            pending: synapse.triggers.clone(),
            crossing: crossing_time(&synapse),
            particles: PARTICLES,
        });
    }
    let ending = world.get::<&ConductanceSynapse>(entity).ok()?;
    let state = world.get::<&SynapseState>(entity).ok()?;
    if ending.conductance <= Microsiemens::ZERO || connection.strength <= 0.0 {
        return None;
    }
    let bound = match state.open > Microsiemens::ZERO {
        true => {
            state.open
                / (ending.conductance * ending_site_scale(world, connection) * connection.strength)
        }
        false => 0.0,
    };
    Some(Release {
        bound: bound.clamp(0.0, 1.0) as f32,
        pending: state.pending.clone(),
        crossing: ending.delay,
        particles: (PARTICLES as f64 * connection.strength.min(1.0)).ceil() as usize,
    })
}

struct Cleft {
    terminal: Vec3,
    terminal_radius: f32,
    particle_radius: f32,
    rounded: bool,
    target: Vec3,
    target_radius: f32,
    contact: Vec3,
}

struct Particle {
    leaving: Vec3,
    landing: Vec3,
    crossing: f32,
}

fn puff(cleft: &Cleft, count: usize, seed: u64) -> Vec<Particle> {
    let (side, up) = cleft.contact.any_orthonormal_pair();
    let depth = if cleft.rounded { 1.0 } else { 0.0 };
    let mut random = StdRng::seed_from_u64(seed);
    let turn: f32 = random.gen();
    let mut arrivals: Vec<usize> = (0..count).collect();
    arrivals.shuffle(&mut random);
    arrivals
        .into_iter()
        .enumerate()
        .map(|(index, arrival)| {
            let order = index as f32;
            let around = TAU * (turn + order * GOLDEN_FRACTION).fract();
            let outwards = side * around.cos() + up * around.sin();
            let share = ((order + random.gen::<f32>()) / count as f32).sqrt();
            let leaving_angle = WIDEST_LEAVING_ANGLE * share;
            let landing_angle = WIDEST_LANDING_ANGLE * share;
            Particle {
                leaving: cleft.terminal
                    + cleft.terminal_radius
                        * (outwards * leaving_angle.sin()
                            - cleft.contact * depth * leaving_angle.cos()),
                landing: cleft.contact * landing_angle.cos() + outwards * landing_angle.sin(),
                crossing: FASTEST_CROSSING
                    + (1.0 - FASTEST_CROSSING) * (arrival + 1) as f32 / count as f32,
            }
        })
        .collect()
}

impl Particle {
    fn flying(&self, cleft: &Cleft, progress: f32, color: Vec3) -> Sphere {
        let from_target = self.leaving - cleft.target;
        let direction = from_target
            .normalize_or_zero()
            .lerp(self.landing, progress)
            .normalize_or_zero();
        let surface = cleft.target_radius + cleft.particle_radius;
        let distance = surface + (from_target.length() - surface) * (1.0 - progress);
        Sphere {
            position: cleft.target + direction * distance,
            color,
            radius: cleft.particle_radius,
            _padding: Default::default(),
        }
    }

    fn landed(&self, cleft: &Cleft, bound: f32, color: Vec3) -> Sphere {
        let radius = cleft.particle_radius * bound;
        Sphere {
            position: cleft.target + self.landing * (cleft.target_radius + radius),
            color,
            radius,
            _padding: Default::default(),
        }
    }
}

fn seed(synapse: Entity, opening: Option<Milliseconds>) -> u64 {
    synapse.to_bits().get() ^ opening.map_or(0, |opening| opening.0.to_bits())
}

#[derive(Default)]
struct Particles {
    flying: Vec<Sphere>,
    landed: Vec<Sphere>,
    halos: Vec<HaloData>,
}

fn shared_halo(puff: &[Sphere], cleft: &Cleft, count: usize, opacity: f32) -> Option<HaloData> {
    if puff.is_empty() {
        return None;
    }
    let center = puff.iter().map(|particle| particle.position).sum::<Vec3>() / puff.len() as f32;
    let spread = puff
        .iter()
        .map(|particle| particle.position.distance(center))
        .fold(0.0, f32::max);
    let size = puff.iter().map(|particle| particle.radius).sum::<f32>();
    let release = (size / (count as f32 * cleft.particle_radius)).min(1.0);
    Some(HaloData {
        position: center,
        radius: spread + HALO_PER_PARTICLE_RADIUS * cleft.particle_radius,
        color: Vec3::ONE.extend(opacity * release),
    })
}

fn particles(
    world: &World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    radii: &HashMap<Entity, f32>,
    palette: &Palette,
    now: Milliseconds,
    landings: &mut Landings,
) -> Particles {
    let display = NeuronDisplay::of(world);
    let mut particles = Particles::default();
    landings.0.retain(|&synapse, _| world.contains(synapse));
    for (&entity, path) in paths.iter().filter(|(_, path)| path.directional) {
        let Ok(connection) = world.get::<&Connection>(entity) else {
            continue;
        };
        let Some(release) = release(world, entity, &connection) else {
            continue;
        };
        let opened = landings.last_opened(entity, &release.pending);
        let crossings = release.crossings(now);
        if release.bound < FAINTEST_BOUND && crossings.is_empty() {
            continue;
        }
        let (Some(&terminal), Ok(target)) =
            (path.points.last(), world.get::<&Position>(connection.to))
        else {
            continue;
        };
        let target = target.position;
        let color = palette.soma(&neuron_type(world, connection.from), 0.0);
        let cleft = Cleft {
            terminal,
            terminal_radius: path.terminal_radius,
            particle_radius: PARTICLE_PER_TERMINAL_RADIUS
                * full_terminal_radius(radii, connection.to),
            rounded: !path.inhibitory,
            target,
            target_radius: radii
                .get(&connection.to)
                .copied()
                .unwrap_or_else(|| soma_radius(world, connection.to, &display)),
            contact: (terminal - target)
                .try_normalize()
                .unwrap_or(-path.terminal_direction),
        };
        let (flying, landed) = (particles.flying.len(), particles.landed.len());
        let flights: Vec<(f32, Vec<Particle>)> = crossings
            .iter()
            .map(|&(opening, through)| {
                let puff = puff(&cleft, release.particles, seed(entity, Some(opening)));
                (through, puff)
            })
            .collect();
        let settled = puff(&cleft, release.particles, seed(entity, opened));
        for (index, mut resting) in settled.iter().enumerate() {
            let mut bound = release.bound;
            for (through, puff) in &flights {
                let particle = &puff[index];
                let progress = through / particle.crossing;
                match progress < 1.0 {
                    true => particles
                        .flying
                        .push(particle.flying(&cleft, progress, color)),
                    false => {
                        bound = 1.0;
                        resting = particle;
                    }
                }
            }
            if bound >= FAINTEST_BOUND {
                particles.landed.push(resting.landed(&cleft, bound, color));
            }
        }
        let puff = [&particles.flying[flying..], &particles.landed[landed..]].concat();
        particles.halos.extend(shared_halo(
            &puff,
            &cleft,
            release.particles,
            palette.transmitter,
        ));
    }
    particles
}

pub fn collect_transmitter(
    world: &World,
    paths: &BTreeMap<Entity, ConnectionPath>,
    radii: &HashMap<Entity, f32>,
    palette: &Palette,
    now: Milliseconds,
    landings: &mut Landings,
) -> Scene {
    let Particles {
        mut flying,
        landed,
        halos,
    } = particles(world, paths, radii, palette, now, landings);
    flying.extend(landed);
    Scene {
        spheres: flying,
        transmitter: halos,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;
    use crate::elements::{
        spawn_cable, spawn_compartment, spawn_dendrite_compartment, spawn_synapse, Element,
    };
    use crate::rendering::cylinders::compartment_radii;
    use crate::rendering::paths::connection_paths;
    use crate::rendering::Theme;
    use crate::simulation::{step_simulation, Engine};
    use glam::Vec3;

    struct Firing {
        engine: Engine,
        time: Milliseconds,
        landings: Landings,
    }

    impl Firing {
        fn step(&mut self, world: &mut World) -> Particles {
            step_simulation(world, &mut self.engine, &mut self.time, 1, None);
            let palette = Theme::Dark.palette();
            let radii = compartment_radii(world);
            particles(
                world,
                &connection_paths(world, &radii, &palette),
                &radii,
                &palette,
                self.time,
                &mut self.landings,
            )
        }
    }

    fn fire(world: &mut World, neuron: Entity) -> Firing {
        let mut firing = quiet();
        firing.engine.sync(world);
        firing.engine.fire(world, neuron);
        firing
    }

    fn quiet() -> Firing {
        Firing {
            engine: Engine::default(),
            time: Milliseconds::ZERO,
            landings: Landings::default(),
        }
    }

    fn synapse_from(element: Element) -> (World, Entity, Entity) {
        let mut world = World::new();
        let source = element.spawn(&mut world, Vec3::ZERO);
        let target = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(10.0, 0.0, 0.0));
        spawn_synapse(&mut world, source, target);
        (world, source, target)
    }

    struct AxonEnding {
        world: World,
        source: Entity,
        ending: Entity,
        target: Entity,
    }

    fn axon_ending(element: Element, strength: f64) -> AxonEnding {
        axon_ending_on(element, strength, |_, target| target)
    }

    fn axon_ending_on_a_dendrite() -> AxonEnding {
        axon_ending_on(Element::ExcitatoryNeuron, 1.0, |world, target| {
            let position = Vec3::new(10.0, 3.0, 0.0);
            let dendrite = spawn_dendrite_compartment(world, position, NeuronType::Excitatory);
            spawn_cable(world, target, dendrite, false);
            dendrite
        })
    }

    fn axon_ending_on(
        element: Element,
        strength: f64,
        site: impl FnOnce(&mut World, Entity) -> Entity,
    ) -> AxonEnding {
        let mut world = World::new();
        let source = element.spawn(&mut world, Vec3::ZERO);
        let target = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::new(10.0, 0.0, 0.0));
        let site = site(&mut world, target);
        let source_type = neuron_type(&world, source);
        let mut previous = source;
        for step in 1..4 {
            let position = Vec3::new(2.0 * step as f32, 0.0, 0.0);
            let compartment = spawn_compartment(&mut world, position, source_type);
            spawn_cable(&mut world, previous, compartment, false);
            previous = compartment;
        }
        let ending = spawn_cable(&mut world, previous, site, true);
        world.get::<&mut Connection>(ending).unwrap().strength = strength;
        AxonEnding {
            world,
            source,
            ending,
            target,
        }
    }

    #[test]
    fn a_quiet_synapse_releases_nothing() {
        let (mut world, _, _) = synapse_from(Element::ExcitatoryNeuron);
        let mut ending = axon_ending(Element::InhibitoryNeuron, 1.0);
        let (mut basic, mut axon) = (quiet(), quiet());
        for _ in 0..2000 {
            for particles in [basic.step(&mut world), axon.step(&mut ending.world)] {
                assert!(particles.flying.is_empty() && particles.landed.is_empty());
            }
        }
    }

    fn cleft() -> Cleft {
        Cleft {
            terminal: Vec3::new(8.0, 0.0, 0.0),
            terminal_radius: 0.5 * NODE_RADIUS,
            particle_radius: PARTICLE_PER_TERMINAL_RADIUS * 0.5 * NODE_RADIUS,
            rounded: true,
            target: Vec3::new(10.0, 0.0, 0.0),
            target_radius: NODE_RADIUS,
            contact: -Vec3::X,
        }
    }

    fn flying_after(through: f32) -> Vec<Sphere> {
        let cleft = cleft();
        puff(&cleft, PARTICLES, 3)
            .into_iter()
            .filter(|particle| through < particle.crossing)
            .map(|particle| particle.flying(&cleft, through / particle.crossing, Vec3::ONE))
            .collect()
    }

    #[test]
    fn released_particles_leave_from_the_side_of_the_terminal_facing_the_target() {
        let cleft = cleft();
        let particles = flying_after(0.0);
        assert_eq!(particles.len(), PARTICLES);
        assert!(particles.iter().all(|particle| {
            let from_terminal = particle.position - cleft.terminal;
            (from_terminal.length() - cleft.terminal_radius).abs() < 1e-5
                && from_terminal.dot(cleft.target - cleft.terminal) > 0.0
        }));
    }

    #[test]
    fn released_particles_land_on_the_target_without_entering_it() {
        let cleft = cleft();
        for step in 0..=100 {
            assert!(flying_after(0.01 * step as f32).iter().all(|particle| {
                particle.position.distance(cleft.target)
                    >= cleft.target_radius + particle.radius - 1e-5
            }));
        }
        assert!(flying_after(1.0).is_empty());
        let mut sideways: f32 = 0.0;
        for particle in puff(&cleft, PARTICLES, 3) {
            let arriving = particle.flying(&cleft, 0.999, Vec3::ONE);
            let landed = particle.landed(&cleft, 1.0, Vec3::ONE);
            assert!(arriving.position.distance(landed.position) < 0.01);
            assert_eq!(arriving.radius, landed.radius);
            let height = landed.position.distance(cleft.target) - cleft.target_radius;
            assert!((height - landed.radius).abs() < 1e-5);
            let offset = (landed.position - cleft.target).reject_from(cleft.contact);
            sideways = sideways.max(offset.length());
        }
        assert!(sideways > 0.5 * cleft.target_radius);
    }

    #[test]
    fn particles_of_a_puff_land_apart() {
        let cleft = cleft();
        for seed in 0..500 {
            for count in 1..=PARTICLES {
                let spots: Vec<Vec3> = puff(&cleft, count, seed)
                    .iter()
                    .map(|particle| particle.landed(&cleft, 1.0, Vec3::ONE).position)
                    .collect();
                for (index, spot) in spots.iter().enumerate() {
                    assert!(spots[index + 1..]
                        .iter()
                        .all(|other| spot.distance(*other) > 2.0 * cleft.particle_radius));
                }
            }
        }
    }

    struct Frame {
        flying: usize,
        spots: Vec<Vec3>,
    }

    fn two_puffs() -> Vec<Frame> {
        let (mut world, source, target) = synapse_from(Element::ExcitatoryNeuron);
        let center = world.get::<&Position>(target).unwrap().position;
        let mut firing = fire(&mut world, source);
        (0..1000)
            .map(|step| {
                if step == 250 {
                    firing.engine.fire(&world, source);
                }
                let particles = firing.step(&mut world);
                Frame {
                    flying: particles.flying.len(),
                    spots: particles
                        .landed
                        .iter()
                        .map(|particle| (particle.position - center).normalize())
                        .collect(),
                }
            })
            .collect()
    }

    fn settled(frames: &[Frame]) -> Vec<&[Frame]> {
        frames
            .split(|frame| frame.flying > 0)
            .filter(|run| run.iter().any(|frame| !frame.spots.is_empty()))
            .collect()
    }

    fn same_spots(first: &[Vec3], second: &[Vec3]) -> bool {
        first.len() == second.len()
            && first
                .iter()
                .zip(second)
                .all(|(spot, other)| spot.distance(*other) < 1e-5)
    }

    #[test]
    fn each_puff_lands_in_new_spots() {
        let frames = two_puffs();
        let runs = settled(&frames);
        assert_eq!(runs.len(), 2);
        let (first, second) = (&runs[0][0].spots, &runs[1][0].spots);
        assert_eq!((first.len(), second.len()), (PARTICLES, PARTICLES));
        assert!(second
            .iter()
            .all(|spot| first.iter().all(|other| spot.distance(*other) > 0.01)));
    }

    #[test]
    fn landed_particles_stay_where_they_landed() {
        let frames = two_puffs();
        let runs = settled(&frames);
        for run in &runs {
            assert!(run
                .iter()
                .filter(|frame| !frame.spots.is_empty())
                .all(|frame| same_spots(&frame.spots, &run[0].spots[..frame.spots.len()])));
        }
        let landed: Vec<Vec3> = runs.iter().flat_map(|run| run[0].spots.clone()).collect();
        assert!(frames
            .iter()
            .flat_map(|frame| &frame.spots)
            .all(|spot| landed.iter().any(|other| spot.distance(*other) < 1e-5)));
    }

    #[test]
    fn particles_take_turns_landing() {
        let counts: Vec<usize> = [0.0, 0.6, 0.8, 1.0]
            .map(|time| flying_after(time).len())
            .to_vec();
        assert!(counts.windows(2).all(|pair| pair[1] < pair[0]));
    }

    #[test]
    fn a_firing_synapse_sends_particles_in_the_color_of_its_source() {
        let palette = Theme::Dark.palette();
        for (element, color) in [
            (Element::ExcitatoryNeuron, palette.excitatory),
            (Element::InhibitoryNeuron, palette.inhibitory),
        ] {
            let (mut world, source, _) = synapse_from(element);
            let mut firing = fire(&mut world, source);
            let seen: Vec<Particles> = (0..1600).map(|_| firing.step(&mut world)).collect();
            assert!(seen
                .iter()
                .flat_map(|particles| particles.flying.iter().chain(&particles.landed))
                .all(|particle| particle.color == color));
            assert!(seen
                .iter()
                .any(|particles| particles.flying.len() == PARTICLES));
            let last = seen.last().unwrap();
            assert!(last.flying.is_empty() && last.landed.is_empty());
        }
    }

    #[test]
    fn a_puff_shares_one_white_halo_that_fades_with_its_particles() {
        let (mut world, source, _) = synapse_from(Element::ExcitatoryNeuron);
        let mut firing = fire(&mut world, source);
        let palette = Theme::Dark.palette();
        let radii = compartment_radii(&world);
        let mut opacities = Vec::new();
        for _ in 0..1600 {
            firing.step(&mut world);
            let paths = connection_paths(&world, &radii, &palette);
            let scene = collect_transmitter(
                &world,
                &paths,
                &radii,
                &palette,
                firing.time,
                &mut firing.landings,
            );
            assert_eq!(
                scene.transmitter.len(),
                usize::from(!scene.spheres.is_empty())
            );
            for halo in &scene.transmitter {
                assert_eq!(halo.color.truncate(), Vec3::ONE);
                assert!(scene.spheres.iter().all(|particle| {
                    halo.position.distance(particle.position) + particle.radius < halo.radius
                }));
                opacities.push(halo.color.w);
            }
        }
        let full = |opacity: f32| (opacity - palette.transmitter).abs() < 1e-5;
        assert!(full(opacities[0]));
        let fading = &opacities[opacities
            .iter()
            .rposition(|&opacity| full(opacity))
            .unwrap()..];
        assert!(fading.len() > 1);
        assert!(fading.windows(2).all(|pair| pair[1] <= pair[0]));
    }

    fn landed_sizes(mut world: World, source: Entity) -> Vec<f32> {
        let mut firing = fire(&mut world, source);
        let seen: Vec<Particles> = (0..2000).map(|_| firing.step(&mut world)).collect();
        let landed = seen
            .iter()
            .rposition(|particles| !particles.flying.is_empty())
            .expect("particles should cross");
        seen[landed + 1..]
            .iter()
            .map(|particles| {
                particles
                    .landed
                    .iter()
                    .map(|particle| particle.radius)
                    .fold(0.0, f32::max)
            })
            .collect()
    }

    #[test]
    fn landed_particles_shrink_as_the_synapse_closes() {
        let mut alpha = synapse_from(Element::ExcitatoryNeuron);
        for synapse in alpha.0.query_mut::<&mut CurrentSynapse>() {
            synapse.alpha_function = true;
        }
        let ending = axon_ending(Element::InhibitoryNeuron, 1.0);
        let basic = synapse_from(Element::ExcitatoryNeuron);
        for (world, source) in [
            (basic.0, basic.1),
            (alpha.0, alpha.1),
            (ending.world, ending.source),
        ] {
            let sizes = landed_sizes(world, source);
            let full = sizes[0];
            assert!(full > 0.95 * PARTICLE_PER_TERMINAL_RADIUS * 0.5 * NODE_RADIUS);
            assert!(sizes.windows(2).all(|pair| pair[1] <= pair[0]));
            assert_eq!(*sizes.last().unwrap(), 0.0);
        }
    }

    #[test]
    fn landed_particles_wait_on_the_target_for_the_rest_of_the_puff() {
        let (mut world, source, _) = synapse_from(Element::ExcitatoryNeuron);
        let mut firing = fire(&mut world, source);
        let mut crossed = false;
        for _ in 0..2000 {
            let particles = firing.step(&mut world);
            if !particles.flying.is_empty() {
                crossed = true;
                assert_eq!(particles.flying.len() + particles.landed.len(), PARTICLES);
            }
        }
        assert!(crossed);
    }

    fn largest_puff(strength: f64) -> (usize, f32) {
        let mut ending = axon_ending(Element::InhibitoryNeuron, strength);
        let mut firing = fire(&mut ending.world, ending.source);
        let mut largest = (0, 0.0);
        for _ in 0..2000 {
            let particles = firing.step(&mut ending.world);
            largest.0 = largest.0.max(particles.flying.len());
            largest.1 = particles
                .flying
                .iter()
                .chain(&particles.landed)
                .map(|particle| particle.radius)
                .fold(largest.1, f32::max);
        }
        largest
    }

    #[test]
    fn a_weaker_axon_ending_sends_fewer_particles_of_the_same_size() {
        let (count, size) = largest_puff(1.0);
        assert_eq!(count, PARTICLES);
        assert_eq!(largest_puff(2.0), (PARTICLES, size));
        assert_eq!(largest_puff(0.3), (3, size));
        assert_eq!(largest_puff(0.0).0, 0);
    }

    fn site_of(world: &World, ending: Entity) -> (Vec3, f32) {
        let site = world.get::<&Connection>(ending).unwrap().to;
        let radius = compartment_radii(world)
            .get(&site)
            .copied()
            .unwrap_or(NODE_RADIUS);
        (world.get::<&Position>(site).unwrap().position, radius)
    }

    fn nearest_leaving_particle(mut ending: AxonEnding) -> f32 {
        let site = site_of(&ending.world, ending.ending);
        let mut firing = fire(&mut ending.world, ending.source);
        for _ in 0..2000 {
            let particles = firing.step(&mut ending.world).flying;
            if !particles.is_empty() {
                return particles
                    .iter()
                    .map(|particle| particle.position.distance(site.0) - site.1)
                    .fold(f32::INFINITY, f32::min);
            }
        }
        panic!("no particles left the terminal");
    }

    #[test]
    fn particles_leave_as_far_from_a_dendrite_as_from_a_soma() {
        let on_soma = nearest_leaving_particle(axon_ending(Element::ExcitatoryNeuron, 1.0));
        let on_dendrite = nearest_leaving_particle(axon_ending_on_a_dendrite());
        assert!(on_soma > 0.4 * NODE_RADIUS, "{on_soma}");
        assert!(
            (on_dendrite - on_soma).abs() < 0.1 * on_soma,
            "{on_dendrite} {on_soma}"
        );
    }

    struct Watched {
        crossing: Vec<usize>,
        fired: Option<usize>,
    }

    fn watch(mut world: World, source: Entity, target: Entity) -> Watched {
        let mut firing = fire(&mut world, source);
        let mut watched = Watched {
            crossing: Vec::new(),
            fired: None,
        };
        for step in 0..2000 {
            if !firing.step(&mut world).flying.is_empty() {
                watched.crossing.push(step);
            }
            let spikes = firing.engine.spikes();
            if watched.fired.is_none() && spikes.iter().any(|spike| spike.entity == target) {
                watched.fired = Some(step);
            }
        }
        watched
    }

    #[test]
    fn a_target_fires_only_after_the_particles_have_landed() {
        let (mut world, source, target) = synapse_from(Element::ExcitatoryNeuron);
        for synapse in world.query_mut::<&mut CurrentSynapse>() {
            synapse.maximum_current = Nanoamperes(30.0);
        }
        let ending = axon_ending(Element::ExcitatoryNeuron, 1.0);
        ending
            .world
            .get::<&mut ConductanceSynapse>(ending.ending)
            .unwrap()
            .conductance *= 20.0;
        for (world, source, target) in [
            (world, source, target),
            (ending.world, ending.source, ending.target),
        ] {
            let watched = watch(world, source, target);
            let fired = watched.fired.expect("one spike should fire the target");
            assert!(!watched.crossing.is_empty());
            assert!(
                watched.crossing.iter().all(|&step| step < fired),
                "{:?} {fired}",
                watched.crossing
            );
        }
    }

    #[test]
    fn particles_cross_an_axon_ending_during_its_synaptic_delay() {
        let ending = axon_ending(Element::ExcitatoryNeuron, 1.0);
        let delay = ending
            .world
            .get::<&ConductanceSynapse>(ending.ending)
            .unwrap()
            .delay;
        let crossing = watch(ending.world, ending.source, ending.target).crossing;
        let steps = crossing.last().unwrap() - crossing.first().unwrap() + 1;
        assert_eq!(steps, crossing.len());
        assert!((DT * steps as f64 - delay).abs() <= DT, "{steps} steps");
    }
}
