use glam::Vec3;
use hecs::Entity;
use visula::{
    CylinderGeometry, CylinderMaterial, Cylinders, Expression, InstanceBuffer, LineGeometry,
    LineMaterial, Lines, MeshPipeline, RenderData, Renderable, ShadowRenderData, SphereGeometry,
    SphereMaterial, Spheres, Torus, TorusGeometry, TorusMaterial,
};

use crate::constants::{NODE_RADIUS, TERMINAL_INSET};
use crate::rendering::colors::{terminal_color, Theme};
use crate::rendering::connections::{collect_meter_wires, connection_being_drawn};
use crate::rendering::cylinders::growing_axon;
use crate::rendering::floor::create_floor;
use crate::rendering::gauges::GaugeRenderer;
use crate::rendering::ghost::Ghost;
use crate::rendering::gpu_types::{ConnectionData, CylinderData, HaloData, Sphere, TorusData};
use crate::rendering::halos::{collect_halos, create_halos};
use crate::rendering::labels::LabelRenderer;
use crate::rendering::lighting::{configure_lighting, set_theme_lighting, shaded};
use crate::rendering::notes::NoteRenderer;
use crate::rendering::scene::collect_scene;
use crate::rendering::selection::{collect_selection, eraser_outline};
use crate::rendering::spike_detector::collect_spike_detectors;
use crate::rendering::voltmeter::collect_voltmeter_traces;

pub struct EditorMarks<'a> {
    pub drawn_connection: &'a [Vec3],
    pub axon: Option<(Entity, Vec3, Vec3)>,
    pub ghost: Option<Ghost>,
    pub selection: Option<Entity>,
    pub eraser: Option<Vec3>,
}

const SHADOW_EXTENT_PER_CAMERA_DISTANCE: f32 = 2.0;
const MIN_SHADOW_EXTENT: f32 = 50.0;
const MAX_SHADOW_EXTENT: f32 = 625.0;

pub struct SceneRenderer {
    theme: Theme,
    floor: MeshPipeline,
    halos: MeshPipeline,
    halo_buffer: InstanceBuffer<HaloData>,
    spheres: Spheres,
    sphere_buffer: InstanceBuffer<Sphere>,
    cylinders: Cylinders,
    cylinder_buffer: InstanceBuffer<CylinderData>,
    rings: Torus,
    ring_buffer: InstanceBuffer<TorusData>,
    connection_lines: Lines,
    connection_spheres: Spheres,
    connection_buffer: InstanceBuffer<ConnectionData>,
    notes: NoteRenderer,
    labels: LabelRenderer,
    gauges: GaugeRenderer,
}

impl SceneRenderer {
    pub fn new(application: &mut visula::Application, theme: Theme) -> SceneRenderer {
        let sphere_buffer = InstanceBuffer::<Sphere>::new(&application.device);
        let connection_buffer = InstanceBuffer::<ConnectionData>::new(&application.device);
        let sphere = sphere_buffer.instance();
        let connection = connection_buffer.instance();

        let spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: sphere.position.clone(),
                radius: sphere.radius,
                color: sphere.color,
            },
            &SphereMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let connection_vector = connection.position_b.clone() - connection.position_a.clone();
        let connection_endpoint = connection.position_a.clone() + connection_vector.clone()
            - connection.directional.clone() * connection_vector.clone()
                / connection_vector.clone().length()
                * TERMINAL_INSET;
        let connection_lines = Lines::new(
            &application.rendering_descriptor(),
            &LineGeometry {
                start: connection.position_a.clone(),
                end: connection_endpoint.clone(),
                width: connection.strength.clone() * 0.3,
                color: connection.start_color.clone(),
            },
            &LineMaterial {
                color: shaded(Expression::InputColor),
            },
        )
        .unwrap();

        let connection_spheres = Spheres::new(
            &application.rendering_descriptor(),
            &SphereGeometry {
                position: connection_endpoint,
                radius: connection.directional.clone() * (0.5 * NODE_RADIUS),
                color: terminal_color().into(),
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
                start_color: cylinder.start_color,
                end_color: cylinder.end_color,
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

        let floor = create_floor(
            &application.rendering_descriptor(),
            &application.device,
            theme.palette().floor,
        )
        .unwrap();
        let halo_buffer = InstanceBuffer::<HaloData>::new(&application.device);
        let halos = create_halos(
            &application.rendering_descriptor(),
            &application.device,
            &halo_buffer,
        )
        .unwrap();
        configure_lighting(application, theme);

        SceneRenderer {
            theme,
            floor,
            halos,
            halo_buffer,
            spheres,
            sphere_buffer,
            cylinders,
            cylinder_buffer,
            rings,
            ring_buffer,
            connection_lines,
            connection_spheres,
            connection_buffer,
            notes: NoteRenderer::new(application),
            labels: LabelRenderer::new(application, theme),
            gauges: GaugeRenderer::new(application),
        }
    }

    pub fn update(
        &mut self,
        application: &mut visula::Application,
        world: &hecs::World,
        time: f64,
        theme: Theme,
        marks: EditorMarks,
    ) {
        if self.theme != theme {
            self.floor = create_floor(
                &application.rendering_descriptor(),
                &application.device,
                theme.palette().floor,
            )
            .unwrap();
            set_theme_lighting(application, theme);
            self.theme = theme;
        }

        let camera = &application.camera_controller.current_transform;
        application.light.shadow_center = camera.center;
        application.light.shadow_extent = (camera.distance * SHADOW_EXTENT_PER_CAMERA_DISTANCE)
            .clamp(MIN_SHADOW_EXTENT, MAX_SHADOW_EXTENT);

        let palette = theme.palette();
        let mut scene = collect_scene(world, &palette, time);
        let axon = marks
            .axon
            .and_then(|(from, start, end)| growing_axon(world, from, start, end, &palette));
        if let Some(axon) = axon {
            scene.extend(axon);
        }
        scene
            .spheres
            .extend(marks.ghost.and_then(|ghost| ghost.sphere(&palette)));
        if let Some(selection) = marks.selection {
            scene.extend(collect_selection(world, selection, &palette));
        }
        if let Some(center) = marks.eraser {
            scene.rings.push(eraser_outline(center, &palette));
        }
        let mut connections = collect_meter_wires(world, &palette);
        connections.extend(connection_being_drawn(marks.drawn_connection, &palette));
        connections.extend(collect_voltmeter_traces(world));
        connections.extend(collect_spike_detectors(world, &palette));

        self.sphere_buffer
            .update(&application.device, &application.queue, &scene.spheres);
        self.cylinder_buffer
            .update(&application.device, &application.queue, &scene.cylinders);
        self.ring_buffer
            .update(&application.device, &application.queue, &scene.rings);
        self.connection_buffer
            .update(&application.device, &application.queue, &connections);
        let mut halos = collect_halos(world, &palette);
        halos.append(&mut scene.transmitter);
        self.halo_buffer
            .update(&application.device, &application.queue, &halos);
        self.notes.update(application, world);
        self.labels.update(application, world, theme);
        self.gauges.update(application, world, &palette);
    }

    pub fn render(&mut self, data: &mut RenderData) {
        self.floor.render(data);
        self.halos.render(data);
        self.spheres.render(data);
        self.cylinders.render(data);
        self.rings.render(data);
        self.connection_lines.render(data);
        self.connection_spheres.render(data);
        self.notes.render(data);
        self.labels.render(data);
        self.gauges.render(data);
    }

    pub fn render_shadow(&mut self, data: &mut ShadowRenderData) {
        self.spheres.render_shadow(data);
        self.cylinders.render_shadow(data);
        self.rings.render_shadow(data);
    }
}
