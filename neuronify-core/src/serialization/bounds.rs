use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use glam::Vec3;
use hecs::World;

use crate::components::*;
use crate::constants::{RESTING_POTENTIAL, VOLTMETER_MEASUREMENTS};
use crate::measurement::firing_rate::{RatePlot, RateSeries};
use crate::measurement::loudspeaker::Loudspeaker;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::visual_input::{ReceptiveField, VisualInput};
use crate::measurement::voltmeter::{VoltageSeries, VoltmeterRange};

pub const POSITIONS: RangeInclusive<f32> = -1e6..=1e6;
pub const MOTIONS: RangeInclusive<f32> = -1e6..=1e6;
pub const FRAME_SIZES: RangeInclusive<f32> = 0.1..=1000.0;
pub const POTENTIALS: RangeInclusive<Millivolts> = Millivolts(-200.0)..=Millivolts(200.0);
pub const VOLTAGES: RangeInclusive<Millivolts> = Millivolts(-1000.0)..=Millivolts(1000.0);
pub const GATES: RangeInclusive<f64> = 0.0..=1.0;
pub const REFRACTORY_PERIODS: RangeInclusive<Milliseconds> =
    Milliseconds(0.0)..=Milliseconds(1000.0);
pub const SOMA_DIAMETERS: RangeInclusive<Micrometres> = Micrometres(1.0)..=Micrometres(1000.0);
pub const CAPACITANCES: RangeInclusive<Nanofarads> = Nanofarads(0.001)..=Nanofarads(10_000.0);
pub const RESISTANCES: RangeInclusive<Megaohms> = Megaohms(0.01)..=Megaohms(100_000.0);
pub const ADAPTATIONS: RangeInclusive<Microsiemens> = Microsiemens(0.0)..=Microsiemens(10.0);
pub const ADAPTATION_TIME_CONSTANTS: RangeInclusive<Milliseconds> =
    Milliseconds(0.0)..=Milliseconds(100_000.0);
pub const CONDUCTANCES: RangeInclusive<Microsiemens> = Microsiemens(0.0)..=Microsiemens(1e9);
pub const BURST_CURRENTS: RangeInclusive<Nanoamperes> = Nanoamperes(0.0)..=Nanoamperes(1000.0);
pub const BURST_DRAINS: RangeInclusive<NanoamperesPerMillisecond> =
    NanoamperesPerMillisecond(0.0)..=NanoamperesPerMillisecond(1000.0);
pub const CURRENTS: RangeInclusive<Nanoamperes> = Nanoamperes(-10_000.0)..=Nanoamperes(10_000.0);
pub const AMPLITUDES: RangeInclusive<Nanoamperes> = Nanoamperes(0.0)..=Nanoamperes(10_000.0);
pub const ALTERNATING_FREQUENCIES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(10_000.0);
pub const SYNAPSE_CURRENTS: RangeInclusive<Nanoamperes> =
    Nanoamperes(-1000.0)..=Nanoamperes(1000.0);
pub const SYNAPSE_TIME_CONSTANTS: RangeInclusive<Milliseconds> =
    Milliseconds(0.01)..=Milliseconds(100_000.0);
pub const SYNAPSE_CONDUCTANCES: RangeInclusive<Microsiemens> =
    Microsiemens(0.0)..=Microsiemens(1000.0);
pub const DELAYS: RangeInclusive<Milliseconds> = Milliseconds(0.0)..=Milliseconds(100_000.0);
pub const SYNAPSE_STATES: RangeInclusive<f64> = -1e9..=1e9;
pub const SPIKE_RATES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(100_000.0);
pub const STRENGTHS: RangeInclusive<f64> = 0.0..=100.0;
pub const CABLE_LENGTHS: RangeInclusive<Micrometres> = Micrometres(0.01)..=Micrometres(1e6);
pub const CABLE_DIAMETERS: RangeInclusive<Micrometres> = Micrometres(0.01)..=Micrometres(10_000.0);
pub const SPECIFIC_CAPACITANCES: RangeInclusive<MicrofaradsPerSquareCentimetre> =
    MicrofaradsPerSquareCentimetre(0.01)..=MicrofaradsPerSquareCentimetre(100.0);
pub const AXIAL_RESISTIVITIES: RangeInclusive<OhmCentimetres> =
    OhmCentimetres(1.0)..=OhmCentimetres(1e6);
pub const CONDUCTANCE_DENSITIES: RangeInclusive<SiemensPerSquareCentimetre> =
    SiemensPerSquareCentimetre(0.0)..=SiemensPerSquareCentimetre(10.0);
pub const SQUID_TEMPERATURES: RangeInclusive<Celsius> = Celsius(0.0)..=Celsius(35.0);
pub const MAMMAL_TEMPERATURES: RangeInclusive<Celsius> = Celsius(20.0)..=Celsius(42.0);
pub const PLOTTED_VOLTAGES: RangeInclusive<Millivolts> = Millivolts(-1000.0)..=Millivolts(1000.0);
pub const RATE_WINDOWS: RangeInclusive<Milliseconds> = Milliseconds(1.0)..=Milliseconds(1e6);
pub const PLOTTED_RATES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(1e6);
pub const TIME_RANGES: RangeInclusive<Milliseconds> = Milliseconds(1.0)..=Milliseconds(1e6);
pub const SENSITIVITIES: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(1e6);
pub const PICTURE_SPEEDS: RangeInclusive<Hertz> = Hertz(0.0)..=Hertz(1000.0);
pub const ANGLES: RangeInclusive<f64> = 0.0..=TAU;
pub const PICTURE_PHASES: RangeInclusive<f64> = 0.0..=1.0;
pub const OFFSETS: RangeInclusive<f64> = -0.5..=0.5;
pub const VOLUMES: RangeInclusive<f64> = 0.0..=1.0;

pub fn bound(world: &mut World) {
    let preparation = Preparation::of(world);
    for position in world.query_mut::<&mut Position>() {
        position.position = bounded_vector(position.position, POSITIONS);
    }
    for dynamics in world.query_mut::<&mut SpatialDynamics>() {
        dynamics.velocity = bounded_vector(dynamics.velocity, MOTIONS);
        dynamics.acceleration = bounded_vector(dynamics.acceleration, MOTIONS);
    }
    for size in world.query_mut::<&mut MeterSize>() {
        let default = MeterSize::default();
        size.width = bounded_f32(size.width, FRAME_SIZES, default.width);
        size.height = bounded_f32(size.height, FRAME_SIZES, default.height);
    }
    for size in world.query_mut::<&mut NoteSize>() {
        let default = NoteSize::default();
        size.width = bounded_f32(size.width, FRAME_SIZES, default.width);
        size.height = bounded_f32(size.height, FRAME_SIZES, default.height);
    }
    for neuron in world.query_mut::<&mut LeakyNeuron>() {
        bound_neuron(neuron);
    }
    for state in world.query_mut::<&mut NodeState>() {
        within(&mut state.voltage, VOLTAGES, RESTING_POTENTIAL);
        bound_gates(&mut state.gates);
        state.since_spike = elapsed(state.since_spike);
    }
    for state in world.query_mut::<&mut CableState>() {
        for voltage in &mut state.voltage {
            within(voltage, VOLTAGES, RESTING_POTENTIAL);
        }
        bound_gates(&mut state.gates);
    }
    for leak in world.query_mut::<&mut LeakCurrent>() {
        let default = LeakCurrent::default();
        within(&mut leak.resistance, RESISTANCES, default.resistance);
    }
    for adaptation in world.query_mut::<&mut AdaptationCurrent>() {
        let default = AdaptationCurrent::default();
        within(&mut adaptation.adaptation, ADAPTATIONS, default.adaptation);
        within(
            &mut adaptation.conductance,
            CONDUCTANCES,
            Microsiemens::ZERO,
        );
        within(
            &mut adaptation.time_constant,
            ADAPTATION_TIME_CONSTANTS,
            default.time_constant,
        );
    }
    for burst in world.query_mut::<&mut BurstCurrent>() {
        let default = BurstCurrent::default();
        within(&mut burst.peak, BURST_CURRENTS, default.peak);
        within(&mut burst.drain, BURST_DRAINS, default.drain);
        within(&mut burst.boost, BURST_CURRENTS, Nanoamperes::ZERO);
    }
    for clamp in world.query_mut::<&mut CurrentClamp>() {
        let default = CurrentClamp::default();
        within(&mut clamp.current_output, CURRENTS, default.current_output);
    }
    for alternating in world.query_mut::<&mut AlternatingCurrent>() {
        let default = AlternatingCurrent::default();
        within(&mut alternating.amplitude, AMPLITUDES, default.amplitude);
        within(
            &mut alternating.frequency,
            ALTERNATING_FREQUENCIES,
            default.frequency,
        );
    }
    for synapse in world.query_mut::<&mut CurrentSynapse>() {
        let default = CurrentSynapse::default();
        within(&mut synapse.tau, SYNAPSE_TIME_CONSTANTS, default.tau);
        within(
            &mut synapse.maximum_current,
            SYNAPSE_CURRENTS,
            default.maximum_current,
        );
        within(&mut synapse.delay, DELAYS, default.delay);
        within(&mut synapse.exponential, SYNAPSE_STATES, 0.0);
        within(&mut synapse.linear, SYNAPSE_STATES, 0.0);
        synapse.triggers.retain(|time| time.is_finite());
    }
    for synapse in world.query_mut::<&mut ConductanceSynapse>() {
        within(
            &mut synapse.conductance,
            SYNAPSE_CONDUCTANCES,
            Microsiemens::ZERO,
        );
        within(&mut synapse.delay, DELAYS, Milliseconds::ZERO);
    }
    for state in world.query_mut::<&mut SynapseState>() {
        within(&mut state.open, CONDUCTANCES, Microsiemens::ZERO);
        state.pending.retain(|time| time.is_finite());
    }
    for events in world.query_mut::<&mut PendingEvents>() {
        for list in [&mut events.adaptation, &mut events.burst, &mut events.fire] {
            list.retain(|time| time.is_finite());
        }
    }
    for generator in world.query_mut::<&mut RegularSpikeGenerator>() {
        let default = RegularSpikeGenerator::default();
        within(&mut generator.frequency, SPIKE_RATES, default.frequency);
    }
    for generator in world.query_mut::<&mut PoissonGenerator>() {
        let default = PoissonGenerator::default();
        within(&mut generator.rate, SPIKE_RATES, default.rate);
    }
    for dynamics in world.query_mut::<&mut GeneratorDynamics>() {
        dynamics.time_since_fire = elapsed(dynamics.time_since_fire);
    }
    for connection in world.query_mut::<&mut Connection>() {
        within(&mut connection.strength, STRENGTHS, 1.0);
    }
    for cable in world.query_mut::<&mut Cable>() {
        within(&mut cable.length, CABLE_LENGTHS, *CABLE_LENGTHS.start());
        within(
            &mut cable.diameter,
            CABLE_DIAMETERS,
            *CABLE_DIAMETERS.start(),
        );
    }
    for membrane in world.query_mut::<&mut Membrane>() {
        bound_membrane(membrane);
    }
    for temperature in world.query_mut::<&mut Temperature>() {
        within(
            &mut temperature.celsius,
            preparation.temperatures(),
            preparation.celsius(),
        );
    }
    for range in world.query_mut::<&mut VoltmeterRange>() {
        let default = VoltmeterRange::default();
        within(&mut range.minimum, PLOTTED_VOLTAGES, default.minimum);
        within(&mut range.maximum, PLOTTED_VOLTAGES, default.maximum);
        if range.minimum >= range.maximum {
            *range = default;
        }
    }
    for series in world.query_mut::<&mut VoltageSeries>() {
        series.bound(VOLTMETER_MEASUREMENTS);
    }
    for plot in world.query_mut::<&mut RatePlot>() {
        let default = RatePlot::default();
        within(&mut plot.window, RATE_WINDOWS, default.window);
        within(&mut plot.kernel_width, RATE_WINDOWS, default.kernel_width);
        within(&mut plot.minimum, PLOTTED_RATES, default.minimum);
        within(&mut plot.maximum, PLOTTED_RATES, default.maximum);
        if plot.minimum >= plot.maximum {
            (plot.minimum, plot.maximum) = (default.minimum, default.maximum);
        }
    }
    for series in world.query_mut::<&mut RateSeries>() {
        series
            .samples
            .retain(|(time, rate)| time.is_finite() && rate.is_finite());
    }
    for detector in world.query_mut::<&mut SpikeDetector>() {
        let default = SpikeDetector::default();
        within(&mut detector.time_range, TIME_RANGES, default.time_range);
        if !detector.time.is_finite() {
            detector.time = Milliseconds::ZERO;
        }
    }
    for train in world.query_mut::<&mut SpikeTrain>() {
        train.times.retain(|time| time.is_finite());
    }
    for input in world.query_mut::<&mut VisualInput>() {
        bound_visual_input(input);
    }
    for speaker in world.query_mut::<&mut Loudspeaker>() {
        within(&mut speaker.volume, VOLUMES, Loudspeaker::default().volume);
    }
}

fn bound_neuron(neuron: &mut LeakyNeuron) {
    let default = LeakyNeuron::default();
    within(&mut neuron.capacitance, CAPACITANCES, default.capacitance);
    within(
        &mut neuron.resting_potential,
        POTENTIALS,
        default.resting_potential,
    );
    within(&mut neuron.threshold, POTENTIALS, default.threshold);
    within(
        &mut neuron.initial_potential,
        POTENTIALS,
        default.initial_potential,
    );
    within(
        &mut neuron.minimum_voltage,
        VOLTAGES,
        default.minimum_voltage,
    );
    within(
        &mut neuron.maximum_voltage,
        VOLTAGES,
        default.maximum_voltage,
    );
    if neuron.minimum_voltage >= neuron.maximum_voltage {
        (neuron.minimum_voltage, neuron.maximum_voltage) =
            (default.minimum_voltage, default.maximum_voltage);
    }
    within(
        &mut neuron.refractory_period,
        REFRACTORY_PERIODS,
        default.refractory_period,
    );
    within(
        &mut neuron.soma_diameter,
        SOMA_DIAMETERS,
        default.soma_diameter,
    );
}

fn bound_membrane(membrane: &mut Membrane) {
    let squid = Membrane::squid();
    within(
        &mut membrane.capacitance,
        SPECIFIC_CAPACITANCES,
        squid.capacitance,
    );
    within(
        &mut membrane.axial_resistivity,
        AXIAL_RESISTIVITIES,
        squid.axial_resistivity,
    );
    within(&mut membrane.leak, CONDUCTANCE_DENSITIES, squid.leak);
    within(&mut membrane.leak_reversal, POTENTIALS, squid.leak_reversal);
    for density in &mut membrane.channels {
        within(
            &mut density.conductance,
            CONDUCTANCE_DENSITIES,
            SiemensPerSquareCentimetre::ZERO,
        );
        within(&mut density.reversal, POTENTIALS, RESTING_POTENTIAL);
    }
}

fn bound_visual_input(input: &mut VisualInput) {
    let default = VisualInput::default();
    within(&mut input.sensitivity, SENSITIVITIES, default.sensitivity);
    within(&mut input.speed, PICTURE_SPEEDS, default.speed);
    within(&mut input.phase, PICTURE_PHASES, default.phase);
    input.pattern_angle = turned(input.pattern_angle);
    match &mut input.field {
        ReceptiveField::Orientation { angle, x, y } => {
            *angle = turned(*angle);
            within(x, OFFSETS, 0.0);
            within(y, OFFSETS, 0.0);
        }
        ReceptiveField::CenterSurround { x, y, .. } => {
            within(x, OFFSETS, 0.0);
            within(y, OFFSETS, 0.0);
        }
        ReceptiveField::Edge { angle } => *angle = turned(*angle),
    }
}

fn bound_gates(gates: &mut [f64]) {
    for gate in gates {
        within(gate, GATES, 0.0);
    }
}

fn within<Q: Quantity>(value: &mut Q, range: RangeInclusive<Q>, fallback: Q) {
    if value.value().is_nan() {
        *value = fallback;
    } else {
        *value.value_mut() = value
            .value()
            .clamp(range.start().value(), range.end().value());
    }
}

fn bounded_f32(value: f32, range: RangeInclusive<f32>, fallback: f32) -> f32 {
    if value.is_nan() {
        fallback
    } else {
        value.clamp(*range.start(), *range.end())
    }
}

fn bounded_vector(vector: Vec3, range: RangeInclusive<f32>) -> Vec3 {
    Vec3::from_array(
        vector
            .to_array()
            .map(|value| bounded_f32(value, range.clone(), 0.0)),
    )
}

fn elapsed(time: Milliseconds) -> Milliseconds {
    if time.0.is_nan() {
        Milliseconds::INFINITY
    } else {
        time.max(Milliseconds::ZERO)
    }
}

fn turned(angle: f64) -> f64 {
    if angle.is_finite() {
        angle.rem_euclid(*ANGLES.end())
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::constants::DEFAULT_SPEED;
    use crate::elements::{spawn_voltmeter, Element};
    use crate::measurement::voltmeter::{measured, RollingWindow, VoltageMeasurement};
    use crate::serialization::{load, read, save};

    fn saved(world: &World) -> Vec<u8> {
        save(world, Milliseconds::ZERO, DEFAULT_SPEED).unwrap()
    }

    fn files_in(folder: &Path, extension: &str) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files.extend(files_in(&path, extension));
            } else if path.extension().is_some_and(|found| found == extension) {
                files.push(path);
            }
        }
        files
    }

    #[test]
    fn the_bounds_leave_every_bundled_simulation_and_neuronify_1_file_as_it_is() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let bundled = files_in(&root.join("examples"), "neuronify");
        let legacy = files_in(&root.join("src/legacy/fixtures"), "nfy");
        assert!(bundled.len() > 20 && legacy.len() > 20);
        for file in &bundled {
            let mut world = read(&std::fs::read(file).unwrap()).unwrap().world;
            let before = saved(&world);
            bound(&mut world);
            assert!(before == saved(&world), "{}", file.display());
        }
        for file in &legacy {
            let contents = std::fs::read_to_string(file).unwrap();
            let simulation = crate::legacy::parse_legacy_nfy(&contents).unwrap();
            let mut world = World::new();
            crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
            let before = saved(&world);
            bound(&mut world);
            assert!(before == saved(&world), "{}", file.display());
        }
    }

    #[test]
    fn a_crafted_file_loads_with_every_value_inside_the_bounds() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        {
            let mut leaky = world.get::<&mut LeakyNeuron>(neuron).unwrap();
            leaky.capacitance = Nanofarads(f64::NAN);
            leaky.threshold = Millivolts(f64::INFINITY);
            leaky.refractory_period = Milliseconds(-3.0);
            (leaky.minimum_voltage, leaky.maximum_voltage) = (Millivolts(40.0), Millivolts(-40.0));
        }
        world.get::<&mut Position>(neuron).unwrap().position = Vec3::new(f32::NAN, 1.0, 1e30);
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::new(10.0, 0.0, 0.0));
        *world.get::<&mut VoltmeterRange>(voltmeter).unwrap() = VoltmeterRange {
            minimum: Millivolts(-60.0),
            maximum: Millivolts(-60.0),
        };
        world.spawn((Temperature {
            celsius: Celsius(1000.0),
        },));

        let loaded = load(&saved(&world)).unwrap().world;

        let leaky = loaded.get::<&LeakyNeuron>(neuron).unwrap();
        let default = LeakyNeuron::default();
        assert_eq!(leaky.capacitance, default.capacitance);
        assert_eq!(leaky.threshold, *POTENTIALS.end());
        assert_eq!(leaky.refractory_period, Milliseconds::ZERO);
        assert_eq!(
            (leaky.minimum_voltage, leaky.maximum_voltage),
            (default.minimum_voltage, default.maximum_voltage)
        );
        assert_eq!(
            loaded.get::<&Position>(neuron).unwrap().position,
            Vec3::new(0.0, 1.0, *POSITIONS.end())
        );
        assert_eq!(
            *loaded.get::<&VoltmeterRange>(voltmeter).unwrap(),
            VoltmeterRange::default()
        );
        assert_eq!(Temperature::of(&loaded).celsius, *SQUID_TEMPERATURES.end());
    }

    #[test]
    fn a_long_history_keeps_its_newest_voltages_and_the_spikes_among_them() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        let voltmeter = spawn_voltmeter(&mut world, neuron, Vec3::new(10.0, 0.0, 0.0));
        let (wire, _) = measured(&world, voltmeter)[0];
        let spacing = Milliseconds(0.1);
        let samples = 5 * VOLTMETER_MEASUREMENTS;
        let mut long = VoltageSeries {
            measurements: RollingWindow::new(samples),
            spike_times: Vec::new(),
        };
        for sample in 0..samples {
            long.measurements.push(VoltageMeasurement {
                voltage: Millivolts(-65.0),
                time: spacing * sample as f64,
            });
        }
        long.spike_times = vec![Milliseconds(1.0), spacing * (samples - 10) as f64];
        *world.get::<&mut VoltageSeries>(wire).unwrap() = long;

        let loaded = load(&saved(&world)).unwrap().world;

        let series = loaded.get::<&VoltageSeries>(wire).unwrap();
        assert_eq!(series.measurements.len(), VOLTMETER_MEASUREMENTS);
        assert_eq!(
            series.measurements.last().unwrap().time,
            spacing * (samples - 1) as f64
        );
        assert_eq!(series.spike_times, [spacing * (samples - 10) as f64]);
    }
}
