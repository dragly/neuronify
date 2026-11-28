use std::ffi::OsStr;
use std::io::Cursor;
use std::path::Path;

const LIST: &str = "assets/third-party-licenses.json";
const ICON: &str = "assets/icon.png";

fn main() {
    println!("cargo::rerun-if-changed={LIST}");
    println!("cargo::rerun-if-changed={ICON}");
    let out = std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts");
    let list = std::fs::read(LIST).expect("the license list should be readable");
    write_compressed(&out, "third-party-licenses.deflate", &list);
    write_compressed(&out, "app-icon.deflate", &icon_pixels());
}

fn icon_pixels() -> Vec<u8> {
    let file = std::fs::read(ICON).expect("the app icon should be readable");
    let mut reader = png::Decoder::new(Cursor::new(file))
        .read_info()
        .expect("the app icon should be a PNG");
    let size = reader
        .output_buffer_size()
        .expect("the app icon should fit in memory");
    let mut rgba = vec![0; size];
    let info = reader
        .next_frame(&mut rgba)
        .expect("the app icon should decode");
    assert_eq!(
        (info.color_type, info.bit_depth, info.width),
        (png::ColorType::Rgba, png::BitDepth::Eight, info.height),
        "the app icon should be a square 8-bit RGBA image"
    );
    rgba.truncate(info.buffer_size());
    rgba
}

fn write_compressed(out: &OsStr, name: &str, data: &[u8]) {
    let compressed = miniz_oxide::deflate::compress_to_vec(data, 9);
    std::fs::write(Path::new(out).join(name), compressed).expect("OUT_DIR should be writable");
}
