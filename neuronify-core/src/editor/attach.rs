use glam::Vec3;
use hecs::{Entity, World};

use crate::components::*;
use crate::constants::*;
use crate::rendering::compartment_radii;
use crate::rendering::cylinders::sheath_radius;

pub const GLIDE_SECONDS: f64 = 0.45;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub from: Entity,
    pub to: Entity,
    pub along: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bead {
    pub from: Entity,
    pub onto: Entity,
    pub at: Vec3,
    pub radius: f32,
}

fn position(world: &World, entity: Entity) -> Option<Vec3> {
    world
        .get::<&Position>(entity)
        .ok()
        .map(|found| found.position)
}

fn is_compartment(world: &World, entity: Entity) -> bool {
    world.satisfies::<&Compartment>(entity).unwrap_or(false)
}

impl Spot {
    pub fn at(&self, world: &World) -> Option<Vec3> {
        Some(position(world, self.from)?.lerp(position(world, self.to)?, self.along))
    }

    pub fn nearest(&self, world: &World) -> Entity {
        let ends = match self.along < 0.5 {
            true => [self.from, self.to],
            false => [self.to, self.from],
        };
        ends.into_iter()
            .find(|&end| is_compartment(world, end))
            .unwrap_or(ends[0])
    }
}

pub fn cable_spot(
    world: &World,
    near: Vec3,
    reach: f32,
    accepts: impl Fn(Entity) -> bool,
) -> Option<Spot> {
    let radii = compartment_radii(world);
    let sheath = sheath_radius(&NeuronDisplay::of(world));
    world
        .query::<(&Connection, Option<&Myelin>)>()
        .with::<&Cable>()
        .iter()
        .filter(|(_, (cable, _))| !cable.directional)
        .filter(|(_, (cable, _))| accepts(cable.from) && accepts(cable.to))
        .filter(|(_, (cable, _))| {
            is_compartment(world, cable.from) || is_compartment(world, cable.to)
        })
        .filter_map(|(_, (cable, myelin))| {
            let start = position(world, cable.from)?;
            let end = position(world, cable.to)?;
            let along = end - start;
            let t = (near - start).dot(along) / along.length_squared().max(f32::MIN_POSITIVE);
            let t = t.clamp(0.0, 1.0);
            let [from_radius, to_radius] = [cable.from, cable.to].map(|end| radii.get(&end));
            let from_radius = from_radius.or(to_radius).copied().unwrap_or(NODE_RADIUS);
            let to_radius = to_radius.copied().unwrap_or(from_radius);
            let mut radius = from_radius + (to_radius - from_radius) * t;
            if myelin.is_some() {
                radius = radius.max(sheath);
            }
            let gap = near.distance(start + t * along) - radius;
            let spot = Spot {
                from: cable.from,
                to: cable.to,
                along: t,
                radius,
            };
            Some((spot, gap))
        })
        .filter(|&(_, gap)| gap < reach)
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(spot, _)| spot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{spawn_cable, spawn_compartment};

    #[test]
    fn a_spot_is_the_nearest_point_along_a_cable_and_its_nearest_compartment() {
        let mut world = World::new();
        let [from, to] = [Vec3::ZERO, Vec3::new(SEGMENT_LENGTH, 0.0, 0.0)]
            .map(|at| spawn_compartment(&mut world, at, NeuronType::Excitatory));
        spawn_cable(&mut world, from, to, false);
        let spot = cable_spot(&world, Vec3::new(0.5, 0.0, 0.4), 1.0, |_| true).unwrap();
        assert_eq!((spot.from, spot.to), (from, to));
        assert!((spot.along - 0.25).abs() < 1e-4);
        assert!(spot.at(&world).unwrap().distance(Vec3::new(0.5, 0.0, 0.0)) < 1e-4);
        assert_eq!(spot.nearest(&world), from);
        let further = cable_spot(&world, Vec3::new(1.5, 0.0, 0.4), 1.0, |_| true).unwrap();
        assert_eq!(further.nearest(&world), to);
        assert!(cable_spot(&world, Vec3::new(1.0, 0.0, 5.0), 1.0, |_| true).is_none());
    }
}
