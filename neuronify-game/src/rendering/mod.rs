pub mod blood_vessels;
pub mod colors;
pub mod game_spheres;
pub mod petri_dish;

pub use blood_vessels::{create_vessel_pipeline, update_vessel_mesh};
pub use game_spheres::collect_game_spheres;
pub use petri_dish::{
    collect_glial_vessel_links, collect_petri_dish, collect_substrate_zone_rings,
    collect_vessel_supply_rings,
};
