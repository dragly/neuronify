use glam::Vec3;
use hecs::{Entity, World};

use crate::components::Label;
use crate::measurement::frame::{measured, Corner, Frame};
use crate::measurement::{firing_rate, loudspeaker, spike_detector, voltmeter, Meter};
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;
use crate::rendering::scene::Scene;
use crate::rendering::shapes::PAD_RADIUS;

impl Meter {
    pub fn rows(self, measured: usize) -> usize {
        match self {
            Meter::RatePlot => 1,
            Meter::Voltmeter | Meter::SpikeDetector | Meter::Loudspeaker => measured,
        }
    }

    pub fn titles(self, world: &World, meter: Entity, frame: &Frame) -> Vec<(String, Vec3)> {
        match self {
            Meter::RatePlot => firing_rate::drawing::titles(world, meter, frame),
            Meter::Voltmeter | Meter::SpikeDetector => row_titles(world, meter, frame),
            Meter::Loudspeaker => Vec::new(),
        }
    }

    pub fn radius(self) -> Option<f32> {
        match self {
            Meter::Loudspeaker => Some(PAD_RADIUS),
            Meter::Voltmeter | Meter::SpikeDetector | Meter::RatePlot => None,
        }
    }
}

fn row_titles(world: &World, meter: Entity, frame: &Frame) -> Vec<(String, Vec3)> {
    let measured = measured(world, meter);
    measured
        .iter()
        .enumerate()
        .filter_map(|(row, (_, target))| {
            let label = world.get::<&Label>(*target).ok()?;
            let corner = frame.row(row, measured.len()).corner(Corner::TopLeft);
            (!label.text.is_empty()).then(|| (label.text.clone(), corner))
        })
        .collect()
}

pub fn shapes(world: &World, palette: &Palette, scene: &mut Scene) {
    for meter in Meter::ALL {
        match meter {
            Meter::Loudspeaker => loudspeaker::drawing::collect_loudspeakers(world, palette, scene),
            Meter::Voltmeter | Meter::SpikeDetector | Meter::RatePlot => {}
        }
    }
}

pub fn lines(world: &World, palette: &Palette) -> Vec<ConnectionData> {
    Meter::ALL
        .into_iter()
        .flat_map(|meter| match meter {
            Meter::Voltmeter => voltmeter::drawing::collect_voltmeter_traces(world, palette),
            Meter::SpikeDetector => {
                spike_detector::drawing::collect_spike_detectors(world, palette)
            }
            Meter::RatePlot => firing_rate::drawing::collect_rate_plots(world, palette),
            Meter::Loudspeaker => Vec::new(),
        })
        .collect()
}
