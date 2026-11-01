pub mod app;
pub mod audio;
pub mod camera;
pub mod components;
pub mod constants;
pub mod dendrites;
pub mod editor;
pub mod elements;
pub mod examples;
pub mod frame_clock;
pub mod gui;
pub mod input;
pub mod landing;
pub mod legacy;
pub mod library;
pub mod measurement;
pub mod rendering;
pub mod serialization;
pub mod settings;
pub mod simulation;
pub mod tools;
#[cfg(target_arch = "wasm32")]
mod web;

pub use app::Neuronify;
pub use components::*;
pub use constants::*;
pub use simulation::{fhn_step, lif_step, run_headless, SpikeRecord};
pub use tools::*;

pub fn run() {
    // On the web, a page can name a file with `?file=<url>`.
    // Neuronify then shows it the way it would look embedded on a web page,
    // with the editor hidden until the reader presses Edit.
    #[cfg(target_arch = "wasm32")]
    let embedded = web::fetch_requested_file();
    #[cfg(not(target_arch = "wasm32"))]
    let embedded = false;
    visula::run(move |application| {
        let mut neuronify = Neuronify::new(application);
        if embedded {
            neuronify.landing = None;
        }
        neuronify
    });
}
