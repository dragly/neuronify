use glam::Vec3;

use crate::components::*;
use crate::measurement::firing_rate::{RatePlot, RateSeries, RATE_HISTORY};
use crate::measurement::voltmeter::{Corner, Frame};
use crate::rendering::colors::*;
use crate::rendering::gpu_types::ConnectionData;

const LINE_STRENGTH: f32 = 0.3;

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

pub fn collect_rate_plots(world: &hecs::World) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for (_, (plot, series, position, size)) in world
        .query::<(&RatePlot, &RateSeries, &Position, &MeterSize)>()
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
        let Some(&(now, _)) = series.samples.back() else {
            continue;
        };
        let span = (plot.maximum - plot.minimum).max(1e-9);
        let at = |(time, rate): (f64, f64)| {
            frame.point(
                (1.0 - (now - time) / RATE_HISTORY) as f32,
                ((rate - plot.minimum) / span).clamp(0.0, 1.0) as f32,
            )
        };
        let samples: Vec<(f64, f64)> = series.samples.iter().copied().collect();
        for pair in samples.windows(2) {
            lines.push(line(at(pair[0]), at(pair[1]), rate_color()));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::spawn_rate_plot;
    use hecs::World;

    #[test]
    fn the_curve_spans_the_last_second_from_the_lowest_to_the_highest_rate() {
        let mut world = World::new();
        let neuron = world.spawn((LeakyDynamics::default(),));
        let plot = spawn_rate_plot(&mut world, neuron, Vec3::ZERO);
        world.get::<&mut RateSeries>(plot).unwrap().samples =
            [(4.0, 0.0), (4.5, 50.0), (5.0, 100.0)]
                .into_iter()
                .collect();
        let frame = Frame::of(&world, plot).unwrap();
        let curve = &collect_rate_plots(&world)[4..];
        assert_eq!(curve.len(), 2);
        assert!(curve[0].position_a.abs_diff_eq(frame.point(0.0, 0.0), 1e-4));
        assert!(curve[0].position_b.abs_diff_eq(frame.point(0.5, 0.5), 1e-4));
        assert!(curve[1].position_b.abs_diff_eq(frame.point(1.0, 1.0), 1e-4));
    }
}
