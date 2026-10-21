use crate::rendering::cylinders::{collect_cylinders, compartment_radii};
use crate::rendering::gpu_types::{CylinderData, Sphere};
use crate::rendering::spheres::collect_spheres;

/// The solid shapes of the 3D view, which the example previews also draw from above.
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub cylinders: Vec<CylinderData>,
}

pub fn collect_scene(world: &hecs::World) -> Scene {
    let radii = compartment_radii(world);
    Scene {
        spheres: collect_spheres(world, &radii),
        cylinders: collect_cylinders(world, &radii),
    }
}
