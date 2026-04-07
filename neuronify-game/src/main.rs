mod app;
mod components;
mod constants;
pub mod map;
mod rendering;
mod simulation;
mod spawning;
mod tools;
mod ui;

use app::GameApp;

fn main() {
    let dev_mode = std::env::args().any(|a| a == "--dev");
    visula::run(move |app| GameApp::new(app, dev_mode));
}
