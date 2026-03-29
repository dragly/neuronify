mod app;
mod components;
mod constants;
mod rendering;
mod simulation;
mod spawning;
mod tools;
mod ui;

use app::GameApp;

fn main() {
    visula::run(GameApp::new);
}
