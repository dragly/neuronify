use hecs::World;

use crate::measurement::{firing_rate, loudspeaker, spike_detector, voltmeter, Meter};
use crate::rendering::colors::Palette;
use crate::rendering::gpu_types::ConnectionData;
use crate::rendering::scene::Scene;

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
