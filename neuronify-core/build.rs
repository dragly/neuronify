use std::path::Path;

const LIST: &str = "assets/third-party-licenses.json";

fn main() {
    println!("cargo::rerun-if-changed={LIST}");
    let list = std::fs::read(LIST).unwrap();
    let compressed = miniz_oxide::deflate::compress_to_vec(&list, 9);
    let out = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(
        Path::new(&out).join("third-party-licenses.deflate"),
        compressed,
    )
    .unwrap();
}
