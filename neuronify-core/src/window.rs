use std::io::Cursor;

use visula::winit::window::{Icon, Window};

const TITLE: &str = "Neuronify";
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

pub fn name(window: &Window) {
    window.set_title(TITLE);
    window.set_window_icon(icon());
}

fn icon() -> Option<Icon> {
    let mut reader = png::Decoder::new(Cursor::new(ICON_PNG)).read_info().ok()?;
    let mut rgba = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut rgba).ok()?;
    Icon::from_rgba(rgba, info.width, info.height).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_decodes() {
        assert!(icon().is_some());
    }
}
