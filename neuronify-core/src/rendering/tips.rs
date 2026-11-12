use glam::Vec3;
use hecs::{Entity, World};

use crate::rendering::colors::Palette;
use crate::rendering::cylinders::path_cylinders;
use crate::rendering::paths::{dangling_path, tip_path, ConnectionPath};
use crate::rendering::scene::Scene;
use crate::rendering::spheres::path_spheres;
use crate::tips::tip;

pub fn collect_tip(
    world: &World,
    palette: &Palette,
    selected: Option<Entity>,
    pulled: Option<Entity>,
) -> Scene {
    let shown = selected
        .filter(|&selected| Some(selected) != pulled)
        .and_then(|selected| tip(world, selected))
        .and_then(|tip| tip_path(world, &tip, palette));
    scene_of(shown)
}

pub fn collect_dangling(world: &World, palette: &Palette, from: Entity, points: &[Vec3]) -> Scene {
    scene_of(dangling_path(world, from, points, palette))
}

fn scene_of(path: Option<ConnectionPath>) -> Scene {
    let mut scene = Scene::default();
    if let Some(path) = path {
        scene.cylinders.extend(path_cylinders(&path));
        scene.spheres.extend(path_spheres(&path));
    }
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;
    use crate::rendering::Theme;

    fn spheres(world: &World, selected: Option<Entity>, pulled: Option<Entity>) -> usize {
        collect_tip(world, &Theme::Light.palette(), selected, pulled)
            .spheres
            .len()
    }

    #[test]
    fn the_tip_shows_on_the_selection_until_it_is_pulled() {
        let mut world = World::new();
        let neuron = Element::ExcitatoryNeuron.spawn(&mut world, Vec3::ZERO);
        assert_eq!(spheres(&world, None, None), 0);
        assert_eq!(spheres(&world, Some(neuron), None), 1);
        assert_eq!(spheres(&world, Some(neuron), Some(neuron)), 0);
    }
}
