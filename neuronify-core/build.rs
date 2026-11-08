use std::path::Path;

const LIST: &str = "assets/third-party-licenses.json";

fn main() {
    println!("cargo::rerun-if-changed={LIST}");
    let list = std::fs::read(LIST).expect("the license list should be readable");
    let compressed = miniz_oxide::deflate::compress_to_vec(&list, 9);
    let out = std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts");
    std::fs::write(
        Path::new(&out).join("third-party-licenses.deflate"),
        compressed,
    )
    .expect("OUT_DIR should be writable");
}
