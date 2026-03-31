use glam::Vec3;

use neuronify_core::Position;

use crate::simulation::setup::PetriDish;

pub fn enforce_petri_boundary(world: &mut hecs::World, dish: &PetriDish) {
    for (_, position) in world.query_mut::<&mut Position>() {
        let offset = position.position - dish.center;
        let offset_2d = Vec3::new(offset.x, 0.0, offset.z);
        let dist = offset_2d.length();
        if dist > dish.radius {
            let dir = offset_2d.normalize();
            position.position =
                dish.center + Vec3::new(dir.x * dish.radius, 0.0, dir.z * dish.radius);
        }
    }
}
