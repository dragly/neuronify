use visula::winit::window::{Icon, Window};

use crate::app_icon;

const TITLE: &str = "Neuronify";

pub fn name(window: &Window) {
    window.set_title(TITLE);
    window.set_window_icon(icon());
    #[cfg(windows)]
    visula::winit::platform::windows::WindowExtWindows::set_taskbar_icon(window, icon());
}

fn icon() -> Option<Icon> {
    let pixels = app_icon::decode()?;
    let size = pixels.size as u32;
    Icon::from_rgba(pixels.rgba, size, size).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_decodes() {
        assert!(icon().is_some());
    }
}
