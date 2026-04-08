mod app;
mod editor;
mod ui;

use app::MapEditorApp;

fn main() {
    visula::run(|application| MapEditorApp::new(application));
}
