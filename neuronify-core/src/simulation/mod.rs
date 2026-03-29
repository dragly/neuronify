pub mod ai;
pub mod fhn;
pub mod game;
pub mod lif;
pub mod spatial;
pub mod stimulation;

pub use fhn::fhn_step;
pub use lif::{lif_step, run_headless, SpikeRecord};
pub use spatial::{apply_spatial_forces, integrate_motion};
pub use stimulation::stimulate_nearby;
