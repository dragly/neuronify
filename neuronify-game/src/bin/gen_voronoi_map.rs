//! One-shot utility: generate the default Voronoi map and save it as JSON.
//! Run with: cargo run --bin gen-voronoi-map

fn main() {
    let model = neuronify_game_lib::voronoi_map::map_gen::generate_scenario_map();
    let json = serde_json::to_string_pretty(&model).expect("serialize");
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("maps")
        .join("voronoi-scenario.json");
    std::fs::write(&path, &json).expect("write");
    println!("Saved {} cells, {} triangles to {}", model.cell_centers.len(), model.triangles.len(), path.display());
}
