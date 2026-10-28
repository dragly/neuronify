use glam::Vec3;

use crate::rendering::colors::Palette;
use crate::rendering::cylinders::{collect_cylinders, compartment_radii};
use crate::rendering::gpu_types::{CylinderData, Sphere, TorusData};
use crate::rendering::paths::connection_paths;
use crate::rendering::shapes::{collect_rings, collect_shapes};
use crate::rendering::spheres::collect_spheres;

/// The solid shapes of the 3D view, which the example previews also draw from above.
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub cylinders: Vec<CylinderData>,
    pub rings: Vec<TorusData>,
}

impl Scene {
    pub fn stroke(&mut self, points: &[Vec3], radius: f32, color: Vec3) {
        for pair in points.windows(2) {
            self.cylinders.push(CylinderData {
                start: pair[0],
                start_radius: radius,
                end: pair[1],
                end_radius: radius,
                color,
                _padding: Default::default(),
            });
        }
        for &position in points {
            self.spheres.push(Sphere {
                position,
                color,
                radius,
                _padding: Default::default(),
            });
        }
    }

    pub fn extend(&mut self, other: Scene) {
        self.spheres.extend(other.spheres);
        self.cylinders.extend(other.cylinders);
        self.rings.extend(other.rings);
    }
}

pub fn collect_scene(world: &hecs::World, palette: &Palette) -> Scene {
    let radii = compartment_radii(world);
    let paths = connection_paths(world, &radii, palette);
    let mut scene = Scene {
        spheres: collect_spheres(world, &radii, &paths, palette),
        cylinders: collect_cylinders(world, &radii, &paths, palette),
        rings: collect_rings(world, palette),
    };
    scene.extend(collect_shapes(world, palette));
    scene
}
