use glam::Vec3;

use crate::rendering::colors::Palette;
use crate::rendering::current_flow::collect_current_flow;
use crate::rendering::cylinders::{collect_cylinders, compartment_radii};
use crate::rendering::gpu_types::{CylinderData, HaloData, Sphere, TorusData};
use crate::rendering::paths::connection_paths;
use crate::rendering::shapes::{collect_rings, collect_shapes};
use crate::rendering::spheres::collect_spheres;
use crate::rendering::transmitter::collect_transmitter;
use crate::rendering::visual_input::collect_visual_input_tiles;

/// The shapes of the 3D view. The example previews draw the solid ones from above.
#[derive(Default)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub cylinders: Vec<CylinderData>,
    pub rings: Vec<TorusData>,
    pub transmitter: Vec<HaloData>,
}

impl Scene {
    pub fn stroke(&mut self, points: &[Vec3], radius: f32, color: Vec3) {
        for pair in points.windows(2) {
            self.cylinders.push(CylinderData {
                start: pair[0],
                start_radius: radius,
                end: pair[1],
                end_radius: radius,
                start_color: color,
                end_color: color,
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
        self.transmitter.extend(other.transmitter);
    }
}

pub fn collect_scene(world: &hecs::World, palette: &Palette, time: f64) -> Scene {
    let radii = compartment_radii(world);
    let paths = connection_paths(world, &radii, palette);
    let mut scene = Scene {
        spheres: collect_spheres(world, &radii, &paths, palette),
        cylinders: collect_cylinders(world, &radii, &paths, palette),
        rings: collect_rings(world, palette),
        transmitter: collect_transmitter(world, &paths, palette),
    };
    scene
        .spheres
        .extend(collect_current_flow(world, &paths, time));
    scene.extend(collect_shapes(world, palette));
    scene.cylinders.extend(collect_visual_input_tiles(world));
    scene
}
