use crate::rendering::colors::Palette;
use crate::rendering::cylinders::{collect_cylinders, compartment_radii};
use crate::rendering::gpu_types::{CylinderData, Sphere, TorusData};
use crate::rendering::paths::connection_paths;
use crate::rendering::shapes::{collect_rings, collect_shapes};
use crate::rendering::spheres::collect_spheres;

/// The solid shapes of the 3D view, which the example previews also draw from above.
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub cylinders: Vec<CylinderData>,
    pub rings: Vec<TorusData>,
}

pub fn collect_scene(world: &hecs::World, palette: &Palette) -> Scene {
    let radii = compartment_radii(world);
    let paths = connection_paths(world, &radii, palette);
    let mut spheres = collect_spheres(world, &radii, &paths, palette);
    let mut cylinders = collect_cylinders(world, &radii, &paths, palette);
    let shapes = collect_shapes(world, palette);
    spheres.extend(shapes.spheres);
    cylinders.extend(shapes.cylinders);
    Scene {
        spheres,
        cylinders,
        rings: collect_rings(world, palette),
    }
}
