const ICON: &str = "../neuronify-core/assets/windows.ico";

fn main() {
    println!("cargo::rerun-if-changed={ICON}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_icon();
    }
}

#[cfg(windows)]
fn embed_icon() {
    winresource::WindowsResource::new()
        .set_icon(ICON)
        .set("ProductName", "Neuronify")
        .set("FileDescription", "Neuronify")
        .compile()
        .expect("the Windows resource compiler should embed the icon");
}

#[cfg(not(windows))]
fn embed_icon() {}
