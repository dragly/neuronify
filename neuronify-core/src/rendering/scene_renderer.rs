use glam::Vec3;
use hecs::Entity;
use visula::{
    ArcGeometry, ArcMaterial, Arcs, CylinderGeometry, CylinderMaterial, Cylinders, Expression,
    InstanceBuffer, LineGeometry, LineMaterial, Lines, MeshPipeline, RenderData, Renderable,
    ShadowRenderData, SphereGeometry, SphereMaterial, Spheres, Torus, TorusGeometry, TorusMaterial,
};

use crate::components::Milliseconds;
use crate::constants::{CONNECTION_WIDTH_PER_STRENGTH, NODE_RADIUS, TERMINAL_INSET};
use crate::editor::Bead;
use crate::rendering::colors::{terminal_color, Theme};
use crate::rendering::connections::{axon_waiting, connection_being_drawn, WireRouter};
use crate::rendering::cylinders::{
    bead_ending, compartment_radii, growing_axon, growing_dendrite, growing_ending,
};
use crate::rendering::floor::create_floor;
use crate::rendering::gauges::GaugeRenderer;
use crate::rendering::ghost::Ghost;
use crate::rendering::gpu_types::{
    ArcData, ConnectionData, CylinderData, HaloData, MeterBackgroundData, Sphere, TorusData,
};
use crate::rendering::halos::{collect_halos, create_halos};
use crate::rendering::labels::{Drawn, LabelRenderer};
use crate::rendering::lighting::{configure_lighting, set_theme_lighting, shaded};
use crate::rendering::meter_backgrounds::{collect_meter_backgrounds, create_meter_backgrounds};
use crate::rendering::notes::NoteRenderer;
use crate::rendering::rate_plot::collect_rate_plots;
use crate::rendering::scene::{collect_scene_along, shown_paths};
use crate::rendering::selection::{collect_selection, eraser_outline};
use crate::rendering::spike_detector::collect_spike_detectors;
use crate::rendering::tips::collect_tip;
use crate::rendering::transmitter::Landings;
use crate::rendering::visual_input::collect_visual_input_lines;
use crate::rendering::voltmeter::collect_voltmeter_traces;

pub struct EditorMarks<'a> {
    pub drawn_connection: &'a [Vec3],
    pub waiting: Option<(Vec3, Vec3)>,
    pub axon: Option<(Entity, Vec3, Vec3)>,
    pub ending: Option<(Entity, Entity)>,
    pub bead: Option<Bead>,
    pub dendrite: Option<(Entity, Vec3, Vec3)>,
    pub ghost: Option<Ghost>,
    pub selection: Option<Entity>,
    pub candidate: Option<Entity>,
    pub eraser: Option<Vec3>,
    pub pulled: Option<Entity>,
}

const SHADOW_EXTENT_PER_CAMERA_DISTANCE: f32 = 2.0;
const MIN_SHADOW_EXTENT: f32 = 50.0;
const MAX_SHADOW_EXTENT: f32 = 625.0;

pub struct SceneRenderer {
    theme: Theme,
    floor: MeshPipeline,
    meter_backgrounds: MeshPipeline,
    meter_background_buffer: InstanceBuffer<MeterBackgroundData>,
    halos: MeshPipeline,
    halo_buffer: InstanceBuffer<HaloData>,
    landings: Landings,
    spheres: Spheres,
    sphere_buffer: InstanceBuffer<Sphere>,
    cylinders: Cylinders,
    cylinder_buffer: InstanceBuffer<CylinderData>,
    rings: Torus,
    ring_buffer: InstanceBuffer<TorusData>,
    wires: Arcs,
    wire_buffer: InstanceBuffer<ArcData>,
    wire_router: WireRouter,
    connection_lines: Lines,
    connection_spheres: Spheres,
    connection_buffer: InstanceBuffer<ConnectionData>,
    notes: NoteRenderer,
    labels: LabelRenderer,
    gauges: GaugeRenderer,
}

impl SceneRenderer {
    pub fn new(
        application: &mut visula::Application,
        theme: Theme,
    ) -> Result<SceneRenderer, visula::error::Error> {
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
        )?;

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
                width: connection.strength.clone() * CONNECTION_WIDTH_PER_STRENGTH,
                color: connection.start_color.clone(),
            },
            &LineMaterial {
                color: shaded(Expression::InputColor),
            },
        )?;

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
        )?;

        let wire_buffer = InstanceBuffer::<ArcData>::new(&application.device);
        let wire = wire_buffer.instance();
        let wires = Arcs::new(
            &application.rendering_descriptor(),
            &ArcGeometry {
                start: wire.start,
                end: wire.end,
                turn: wire.turn,
                width: CONNECTION_WIDTH_PER_STRENGTH.into(),
                normal: Vec3::Y.into(),
                color: wire.color,
            },
            &ArcMaterial {
                color: shaded(Expression::InputColor),
            },
        )?;

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
        )?;

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
        )?;

        let floor = create_floor(
            &application.rendering_descriptor(),
            &application.device,
            theme.palette().floor,
        )?;
        let meter_background_buffer =
            InstanceBuffer::<MeterBackgroundData>::new(&application.device);
        let meter_backgrounds = create_meter_backgrounds(
            &application.rendering_descriptor(),
            &application.device,
            &meter_background_buffer,
            &theme.palette(),
        )?;
        let halo_buffer = InstanceBuffer::<HaloData>::new(&application.device);
        let halos = create_halos(
            &application.rendering_descriptor(),
            &application.device,
            &halo_buffer,
        )?;
        configure_lighting(application, theme);

        Ok(SceneRenderer {
            theme,
            floor,
            meter_backgrounds,
            meter_background_buffer,
            halos,
            halo_buffer,
            landings: Landings::default(),
            spheres,
            sphere_buffer,
            cylinders,
            cylinder_buffer,
            rings,
            ring_buffer,
            wires,
            wire_buffer,
            wire_router: WireRouter::default(),
            connection_lines,
            connection_spheres,
            connection_buffer,
            notes: NoteRenderer::new(application)?,
            labels: LabelRenderer::new(application, theme)?,
            gauges: GaugeRenderer::new(application)?,
        })
    }

    pub fn update(
        &mut self,
        application: &mut visula::Application,
        world: &hecs::World,
        time: Milliseconds,
        theme: Theme,
        marks: EditorMarks,
    ) {
        if self.theme != theme {
            match create_floor(
                &application.rendering_descriptor(),
                &application.device,
                theme.palette().floor,
            ) {
                Ok(floor) => self.floor = floor,
                Err(error) => log::error!("Kept the floor of the last theme: {error}"),
            }
            match create_meter_backgrounds(
                &application.rendering_descriptor(),
                &application.device,
                &self.meter_background_buffer,
                &theme.palette(),
            ) {
                Ok(backgrounds) => self.meter_backgrounds = backgrounds,
                Err(error) => log::error!("Kept the meter backgrounds of the last theme: {error}"),
            }
            set_theme_lighting(application, theme);
            self.theme = theme;
        }

        let camera = &application.camera_controller.current_transform;
        application.light.shadow_center = camera.center;
        application.light.shadow_extent = (camera.distance * SHADOW_EXTENT_PER_CAMERA_DISTANCE)
            .clamp(MIN_SHADOW_EXTENT, MAX_SHADOW_EXTENT);

        let palette = theme.palette();
        let radii = compartment_radii(world);
        let paths = shown_paths(world, &radii, &palette);
        let mut scene =
            collect_scene_along(world, &radii, &paths, &palette, time, &mut self.landings);
        scene.extend(collect_tip(world, &palette, marks.selection, marks.pulled));
        let axon = match (marks.bead, marks.ending) {
            (Some(bead), _) => bead_ending(world, &bead, &palette),
            (None, Some((from, to))) => growing_ending(world, from, to, &palette),
            (None, None) => marks
                .axon
                .and_then(|(from, start, end)| growing_axon(world, from, start, end, &palette)),
        };
        let dendrite = marks
            .dendrite
            .and_then(|(from, start, end)| growing_dendrite(world, from, start, end, &palette));
        for grown in axon.into_iter().chain(dendrite) {
            scene.extend(grown);
        }
        scene
            .spheres
            .extend(marks.ghost.and_then(|ghost| ghost.sphere(&palette)));
        for marked in marks.selection.into_iter().chain(marks.candidate) {
            scene.extend(collect_selection(world, marked, &palette));
        }
        if let Some(center) = marks.eraser {
            scene.rings.push(eraser_outline(center, &palette));
        }
        let wires = self
            .wire_router
            .collect(world, &palette, crate::input::now());
        let mut connections = connection_being_drawn(marks.drawn_connection, &palette);
        connections.extend(
            marks
                .waiting
                .and_then(|(start, end)| axon_waiting(start, end, &palette)),
        );
        connections.extend(collect_voltmeter_traces(world, &palette));
        connections.extend(collect_spike_detectors(world, &palette));
        connections.extend(collect_rate_plots(world, &palette));
        connections.extend(collect_visual_input_lines(world));

        self.sphere_buffer
            .update(&application.device, &application.queue, &scene.spheres);
        self.cylinder_buffer
            .update(&application.device, &application.queue, &scene.cylinders);
        self.ring_buffer
            .update(&application.device, &application.queue, &scene.rings);
        self.wire_buffer
            .update(&application.device, &application.queue, &wires);
        self.connection_buffer
            .update(&application.device, &application.queue, &connections);
        self.meter_background_buffer.update(
            &application.device,
            &application.queue,
            &collect_meter_backgrounds(world),
        );
        let mut halos = collect_halos(world, &palette);
        halos.append(&mut scene.transmitter);
        self.halo_buffer
            .update(&application.device, &application.queue, &halos);
        self.notes.update(application, world);
        let drawn = Drawn {
            radii: &radii,
            paths: &paths,
            wires: &wires,
        };
        self.labels.update(application, world, theme, &drawn);
        self.gauges.update(application, world, &palette);
    }

    pub fn render(&mut self, data: &mut RenderData) {
        self.floor.render(data);
        self.meter_backgrounds.render(data);
        self.halos.render(data);
        self.spheres.render(data);
        self.cylinders.render(data);
        self.rings.render(data);
        self.wires.render(data);
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
