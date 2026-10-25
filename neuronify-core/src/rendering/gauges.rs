//! Gauges for how strongly the adaptation and burst currents act on a neuron right now:
//! a ring around the soma that fills for adaptation, and a bar beside it for bursting.
//! They face the camera and cast no shadows.

use glam::{Quat, Vec3};
use visula::{
    CylinderGeometry, CylinderMaterial, Cylinders, Expression, InstanceBuffer, RenderData,
    Renderable, SphereGeometry, SphereMaterial, Spheres, Torus, TorusGeometry, TorusMaterial,
};

use crate::components::*;
use crate::constants::NODE_RADIUS;
use crate::rendering::colors::*;
use crate::rendering::gpu_types::{CylinderData, Sphere, TorusData};
use crate::rendering::lighting::shaded;
use crate::rendering::shapes::is_inhibitory;

/// Adaptation conductance, in spikes' worth, at which the ring is about two thirds full.
const ADAPTATION_SCALE: f64 = 2.0;
/// How far the empty part of a gauge goes from the colour of the neuron towards the floor.
const TRACK_MIX: f32 = 0.7;
/// Gauges float this far towards the camera, above the tubes around the soma.
const LIFT: f32 = 0.5 * NODE_RADIUS;
const RING_RADIUS: f32 = 1.32 * NODE_RADIUS;
const RING_THICKNESS: f32 = 0.075 * NODE_RADIUS;
const BAR_LENGTH: f32 = 1.8 * NODE_RADIUS;
const BAR_THICKNESS: f32 = 0.14 * NODE_RADIUS;

#[derive(Default)]
struct Gauges {
    spheres: Vec<Sphere>,
    cylinders: Vec<CylinderData>,
    rings: Vec<TorusData>,
}

impl Gauges {
    fn stroke(&mut self, points: &[Vec3], radius: f32, color: Vec3) {
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
}

pub struct GaugeRenderer {
    spheres: Spheres,
    sphere_buffer: InstanceBuffer<Sphere>,
    cylinders: Cylinders,
    cylinder_buffer: InstanceBuffer<CylinderData>,
    rings: Torus,
    ring_buffer: InstanceBuffer<TorusData>,
}

impl GaugeRenderer {
    pub fn new(application: &visula::Application) -> Self {
        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: sphere.position,
                radius: sphere.radius,
                color: sphere.color,
            },
            &SphereMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let cylinder_buffer = InstanceBuffer::<CylinderData>::new(&application.device);
        let cylinder = cylinder_buffer.instance();
        let cylinders = Cylinders::new(
            &application.rendering_descriptor(),
            &CylinderGeometry {
                start: cylinder.start,
                end: cylinder.end,
                start_radius: cylinder.start_radius,
                end_radius: cylinder.end_radius,
                color: cylinder.color,
            },
            &CylinderMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let ring_buffer = InstanceBuffer::<TorusData>::new(&application.device);
        let ring = ring_buffer.instance();
        let rings = Torus::new(
            &application.rendering_descriptor(),
            &TorusGeometry {
                position: ring.position,
                major_radius: ring.major_radius,
                minor_radius: ring.minor_radius,
                rotation: ring.rotation,
                color: ring.color,
            },
            &TorusMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        Self {
            spheres,
            sphere_buffer,
            cylinders,
            cylinder_buffer,
            rings,
            ring_buffer,
        }
    }

    pub fn update(
        &mut self,
        application: &visula::Application,
        world: &hecs::World,
        palette: &Palette,
    ) {
        let camera = &application.camera_controller.current_transform;
        let gauges = collect_gauges(world, palette, camera.position(), camera.forward, camera.up);
        self.sphere_buffer
            .update(&application.device, &application.queue, &gauges.spheres);
        self.cylinder_buffer
            .update(&application.device, &application.queue, &gauges.cylinders);
        self.ring_buffer
            .update(&application.device, &application.queue, &gauges.rings);
    }

    /// There is no `render_shadow`, so gauges cast no shadows.
    pub fn render(&mut self, data: &mut RenderData) {
        self.spheres.render(data);
        self.cylinders.render(data);
        self.rings.render(data);
    }
}

fn adaptation_level(adaptation: &AdaptationCurrent) -> f32 {
    let scale = ADAPTATION_SCALE * adaptation.adaptation.max(f64::MIN_POSITIVE);
    (1.0 - (-adaptation.conductance / scale).exp()) as f32
}

fn burst_level(burst: &BurstCurrent) -> f32 {
    (burst.boost / burst.peak.max(f64::MIN_POSITIVE)).clamp(0.0, 1.0) as f32
}

fn collect_gauges(
    world: &hecs::World,
    palette: &Palette,
    eye: Vec3,
    forward: Vec3,
    view_up: Vec3,
) -> Gauges {
    let screen_right = forward.cross(view_up).normalize();
    let mut gauges = Gauges::default();
    for (entity, position) in world.query::<&Position>().with::<&LeakyNeuron>().iter() {
        let adaptation = world
            .get::<&AdaptationCurrent>(entity)
            .ok()
            .map(|adaptation| adaptation_level(&adaptation));
        let burst = world
            .get::<&BurstCurrent>(entity)
            .ok()
            .map(|burst| burst_level(&burst));
        if adaptation.is_none() && burst.is_none() {
            continue;
        }
        let neuron_type = if is_inhibitory(world, entity) {
            NeuronType::Inhibitory
        } else {
            NeuronType::Excitatory
        };
        let track = palette
            .soma(&neuron_type, 0.0)
            .lerp(palette.floor, TRACK_MIX);
        let normal = (eye - position.position).normalize();
        let right = (screen_right - normal * screen_right.dot(normal)).normalize();
        let up = normal.cross(right);
        let center = position.position + normal * LIFT;
        let front = normal * 0.04;

        if let Some(level) = adaptation {
            gauges.rings.push(TorusData {
                position: center,
                major_radius: RING_RADIUS,
                rotation: Quat::from_rotation_arc(Vec3::Y, normal),
                color: track,
                minor_radius: RING_THICKNESS,
            });
            if level > 0.005 {
                let steps = (64.0 * level).ceil() as usize;
                let points: Vec<Vec3> = (0..=steps)
                    .map(|step| {
                        let angle = std::f32::consts::TAU * level * step as f32 / steps as f32;
                        center + front + (up * angle.cos() + right * angle.sin()) * RING_RADIUS
                    })
                    .collect();
                gauges.stroke(&points, 1.3 * RING_THICKNESS, adaptation_gauge_color());
            }
        }

        if let Some(level) = burst {
            let offset = if adaptation.is_some() { 1.75 } else { 1.6 } * NODE_RADIUS;
            let bottom = center + right * offset - up * 0.5 * BAR_LENGTH;
            gauges.stroke(&[bottom, bottom + up * BAR_LENGTH], BAR_THICKNESS, track);
            if level > 0.005 {
                gauges.stroke(
                    &[bottom + front, bottom + front + up * BAR_LENGTH * level],
                    1.2 * BAR_THICKNESS,
                    burst_gauge_color(),
                );
            }
        }
    }
    gauges
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neuron(world: &mut hecs::World) -> hecs::Entity {
        world.spawn((
            Position {
                position: Vec3::ZERO,
            },
            LeakyNeuron::default(),
            NeuronType::Excitatory,
        ))
    }

    #[test]
    fn adaptation_gets_a_ring_and_bursting_a_bar() {
        let mut world = hecs::World::new();
        neuron(&mut world);
        let adaptive = neuron(&mut world);
        world
            .insert_one(adaptive, AdaptationCurrent::default())
            .unwrap();
        let bursting = neuron(&mut world);
        world.insert_one(bursting, BurstCurrent::default()).unwrap();

        let gauges = collect_gauges(
            &world,
            &Theme::Dark.palette(),
            Vec3::new(0.0, 50.0, 0.0),
            Vec3::NEG_Y,
            Vec3::X,
        );
        assert_eq!(gauges.rings.len(), 1);
        assert_eq!(gauges.cylinders.len(), 1);
    }

    #[test]
    fn gauges_fill_with_the_current() {
        let mut adaptation = AdaptationCurrent::default();
        assert_eq!(adaptation_level(&adaptation), 0.0);
        adaptation.conductance = 4.0 * adaptation.adaptation;
        assert!(adaptation_level(&adaptation) > 0.8);

        let mut burst = BurstCurrent::default();
        assert_eq!(burst_level(&burst), 0.0);
        burst.boost = burst.peak;
        assert_eq!(burst_level(&burst), 1.0);
    }
}
