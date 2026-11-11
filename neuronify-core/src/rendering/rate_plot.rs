use glam::Vec3;

use crate::components::*;
use crate::constants::CURVE_LIFT;
use crate::measurement::firing_rate::{RatePlot, RateSeries, RATE_HISTORY};
use crate::measurement::voltmeter::{measured, Corner, Frame};
use crate::rendering::colors::*;
use crate::rendering::gpu_types::ConnectionData;
use hecs::Entity;

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

fn curve_color(world: &hecs::World, plot: Entity, palette: &Palette) -> Vec3 {
    let colors: Vec<Vec3> = measured(world, plot)
        .into_iter()
        .map(|(_, neuron)| palette.measured(world, neuron))
        .collect();
    match colors.len() {
        0 => palette.neutral,
        count => colors.into_iter().sum::<Vec3>() / count as f32,
    }
}

pub fn collect_rate_plots(world: &hecs::World, palette: &Palette) -> Vec<ConnectionData> {
    let mut lines = Vec::new();
    for (entity, (plot, series, position, size)) in world
        .query::<(Entity, (&RatePlot, &RateSeries, &Position, &MeterSize))>()
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
        let span = (plot.maximum - plot.minimum).max(Hertz(1e-9));
        let at = |(time, rate): (Milliseconds, Hertz)| {
            frame.point(
                (1.0 - (now - time) / RATE_HISTORY) as f32,
                ((rate - plot.minimum) / span).clamp(0.0, 1.0) as f32,
            ) + Vec3::Y * CURVE_LIFT
        };
        let color = curve_color(world, entity, palette);
        let samples: Vec<(Milliseconds, Hertz)> = series.samples.iter().copied().collect();
        for pair in samples.windows(2) {
            lines.push(line(at(pair[0]), at(pair[1]), color));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_rate_plot, spawn_rate_plot_wire};
    use hecs::World;

    #[test]
    fn the_curve_spans_the_last_second_from_the_lowest_to_the_highest_rate() {
        let mut world = World::new();
        let neuron = world.spawn((NodeState::default(),));
        let plot = spawn_rate_plot(&mut world, neuron, Vec3::ZERO);
        world.get::<&mut RateSeries>(plot).unwrap().samples =
            [(4000.0, 0.0), (4500.0, 50.0), (5000.0, 100.0)]
                .into_iter()
                .map(|(time, rate)| (Milliseconds(time), Hertz(rate)))
                .collect();
        let frame = Frame::of(&world, plot).unwrap();
        let curve = &collect_rate_plots(&world, &Theme::default().palette())[4..];
        let over = |right, up| frame.point(right, up) + Vec3::Y * CURVE_LIFT;
        assert_eq!(curve.len(), 2);
        assert!(curve[0].position_a.abs_diff_eq(over(0.0, 0.0), 1e-4));
        assert!(curve[0].position_b.abs_diff_eq(over(0.5, 0.5), 1e-4));
        assert!(curve[1].position_b.abs_diff_eq(over(1.0, 1.0), 1e-4));
    }

    #[test]
    fn the_curve_takes_the_colour_of_its_neurons_and_blends_mixed_types() {
        let mut world = World::new();
        let excitatory = world.spawn((NodeState::default(), NeuronType::Excitatory));
        let inhibitory = world.spawn((NodeState::default(), NeuronType::Inhibitory));
        let plot = spawn_rate_plot(&mut world, excitatory, Vec3::ZERO);
        let palette = Theme::Dark.palette();
        assert_eq!(curve_color(&world, plot, &palette), palette.excitatory);
        spawn_rate_plot_wire(&mut world, inhibitory, plot);
        assert_eq!(
            curve_color(&world, plot, &palette),
            (palette.excitatory + palette.inhibitory) / 2.0
        );
    }
}
