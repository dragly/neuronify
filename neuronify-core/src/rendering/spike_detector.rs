use glam::Vec3;

use crate::components::*;
use crate::measurement::spike_detector::{SpikeDetector, SpikeTrain};
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::colors::*;
use crate::rendering::gpu_types::ConnectionData;

const LINE_STRENGTH: f32 = 0.3;
const TICK_MARGIN: f32 = 0.15;

fn line(position_a: Vec3, position_b: Vec3, color: Vec3) -> ConnectionData {
    ConnectionData {
        position_a,
        position_b,
        strength: LINE_STRENGTH,
        directional: 0.0,
        start_color: color,
        end_color: color,
        _padding: Default::default(),
    }
}

pub fn collect_spike_detectors(world: &hecs::World, palette: &Palette) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for (detector, (spike_detector, position, size)) in world
        .query::<(&SpikeDetector, &Position, &MeterSize)>()
        .iter()
    {
        let frame = Frame::new(position.position, size);
        let corners = [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomRight,
            Corner::BottomLeft,
        ]
        .map(|corner| frame.corner(corner));
        for (index, &corner) in corners.iter().enumerate() {
            lines.push(line(
                corner,
                corners[(index + 1) % corners.len()],
                frame_color(),
            ));
        }
        let start = spike_detector.time - spike_detector.time_range;
        let rows = measured(world, detector);
        for (index, (wire, neuron)) in rows.iter().enumerate() {
            let row = frame.row(index, rows.len());
            let Ok(train) = world.get::<&SpikeTrain>(*wire) else {
                continue;
            };
            let color = if neuron_type(world, *neuron) == NeuronType::Inhibitory {
                palette.inhibitory
            } else {
                palette.excitatory
            };
            for &time in &train.times {
                let right = ((time - start) / spike_detector.time_range) as f32;
                if !(0.0..=1.0).contains(&right) {
                    continue;
                }
                lines.push(line(
                    row.point(right, 1.0 - TICK_MARGIN),
                    row.point(right, TICK_MARGIN),
                    color,
                ));
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_spike_detector, spawn_spike_detector_wire};
    use hecs::World;

    #[test]
    fn draws_a_tick_in_the_row_of_each_wired_neuron() {
        let mut world = World::new();
        let excitatory = world.spawn((LeakyDynamics::default(), NeuronType::Excitatory));
        let inhibitory = world.spawn((LeakyDynamics::default(), NeuronType::Inhibitory));
        let detector = spawn_spike_detector(&mut world, excitatory, Vec3::ZERO);
        spawn_spike_detector_wire(&mut world, inhibitory, detector);
        *world.get::<&mut SpikeDetector>(detector).unwrap() = SpikeDetector {
            time_range: 0.1,
            time: 1.0,
        };
        let rows = measured(&world, detector);
        for ((wire, _), time) in rows.iter().zip([0.95, 1.0]) {
            world
                .get::<&mut SpikeTrain>(*wire)
                .unwrap()
                .times
                .push_back(time);
        }
        let palette = Theme::default().palette();
        let ticks = &collect_spike_detectors(&world, &palette)[4..];
        let frame = Frame::of(&world, detector).unwrap();
        let expected = [
            (frame.row(0, 2), 0.5, palette.excitatory),
            (frame.row(1, 2), 1.0, palette.inhibitory),
        ];
        assert_eq!(ticks.len(), expected.len());
        for (tick, (row, right, color)) in ticks.iter().zip(expected) {
            assert!(tick
                .position_a
                .abs_diff_eq(row.point(right, 1.0 - TICK_MARGIN), 1e-4));
            assert!(tick
                .position_b
                .abs_diff_eq(row.point(right, TICK_MARGIN), 1e-4));
            assert_eq!(tick.start_color, color);
        }
    }
}
