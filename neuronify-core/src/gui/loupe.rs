use bytemuck::{Pod, Zeroable};
use egui::{Context, Id, LayerId, Order, Pos2, Rect, Stroke, Vec2};
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

use super::style::{Chrome, MARGIN};
use crate::rendering::Theme;

pub const LOUPE_DELAY: f64 = 0.15;
const RADIUS: f32 = 56.0;
const ZOOM: f32 = 1.5;
const CLEARANCE: f32 = 64.0;
const SHADOW_DROP: f32 = 3.0;
const CROSSHAIR_RADIUS: f32 = 4.0;
const THIN: f32 = 1.0;
const RIM_WIDTH: f32 = 1.5;
const ATTACHED_RIM_WIDTH: f32 = 3.0;
const ARC_POINTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoupeCorner {
    Left,
    Right,
}

impl LoupeCorner {
    fn center(self, screen: Rect) -> Pos2 {
        let inset = MARGIN + RADIUS;
        match self {
            LoupeCorner::Left => screen.left_top() + Vec2::splat(inset),
            LoupeCorner::Right => screen.right_top() + Vec2::new(-inset, inset),
        }
    }

    fn other(self) -> LoupeCorner {
        match self {
            LoupeCorner::Left => LoupeCorner::Right,
            LoupeCorner::Right => LoupeCorner::Left,
        }
    }

    pub fn next(current: Option<LoupeCorner>, finger: Pos2, screen: Rect) -> LoupeCorner {
        let away = match finger.x > screen.center().x {
            true => LoupeCorner::Left,
            false => LoupeCorner::Right,
        };
        let corner = current.unwrap_or(away);
        let crowded = corner.center(screen).distance(finger) < RADIUS + CLEARANCE;
        match crowded {
            true => corner.other(),
            false => corner,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    center: [f32; 2],
    source: [f32; 2],
    size: [f32; 2],
    radius: f32,
    zoom: f32,
    encode_srgb: u32,
    padding: [u32; 3],
}

struct Resources {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
}

impl Resources {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Resources {
        let shader = device.create_shader_module(wgpu::include_wgsl!("loupe.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("loupe"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("loupe"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("loupe"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Resources {
            pipeline,
            layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("loupe"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            uniforms: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("loupe"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            bind_group: None,
        }
    }
}

pub fn prepare_loupe(application: &mut visula::Application) {
    let device = &application.device;
    let resources = &mut application.egui_renderer.renderer.callback_resources;
    let loupe = match resources.get_mut::<Resources>() {
        Some(loupe) => loupe,
        None => {
            resources.insert(Resources::new(device, application.config.format));
            resources.get_mut::<Resources>().unwrap()
        }
    };
    loupe.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("loupe"),
        layout: &loupe.layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&application.post_processor.hdr_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&loupe.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: loupe.uniforms.as_entire_binding(),
            },
        ],
    }));
}

struct Callback {
    uniforms: Uniforms,
}

impl CallbackTrait for Callback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(loupe) = resources.get::<Resources>() {
            queue.write_buffer(&loupe.uniforms, 0, bytemuck::bytes_of(&self.uniforms));
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let Some(loupe) = resources.get::<Resources>() else {
            return;
        };
        let Some(bind_group) = &loupe.bind_group else {
            return;
        };
        pass.set_pipeline(&loupe.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}

pub struct LoupeView {
    pub corner: LoupeCorner,
    pub finger: Pos2,
    pub screen: Rect,
    pub srgb_target: bool,
    pub attached: bool,
    pub progress: Option<f32>,
    pub flash: f32,
}

fn arc(center: Pos2, radius: f32, fraction: f32) -> Vec<Pos2> {
    let points = (ARC_POINTS as f32 * fraction).ceil().max(1.0) as usize;
    (0..=points)
        .map(|index| {
            let angle = std::f32::consts::TAU * fraction * index as f32 / points as f32;
            center + radius * Vec2::new(angle.sin(), -angle.cos())
        })
        .collect()
}

pub fn paint_loupe(context: &Context, theme: Theme, view: LoupeView) {
    let chrome = Chrome::of(theme);
    let painter = context.layer_painter(LayerId::new(Order::Foreground, Id::new("loupe")));
    let center = view.corner.center(view.screen);
    let pixels = context.pixels_per_point();
    let uniforms = Uniforms {
        center: (center.to_vec2() * pixels).into(),
        source: (view.finger.to_vec2() * pixels).into(),
        size: (view.screen.size() * pixels).into(),
        radius: RADIUS * pixels,
        zoom: ZOOM,
        encode_srgb: u32::from(!view.srgb_target),
        padding: [0; 3],
    };
    painter.circle_filled(center + Vec2::Y * SHADOW_DROP, RADIUS + 2.0, chrome.shadow);
    let bounds = Rect::from_center_size(center, Vec2::splat(2.0 * RADIUS));
    painter.add(egui_wgpu::Callback::new_paint_callback(
        bounds,
        Callback { uniforms },
    ));
    painter.circle_stroke(
        center,
        CROSSHAIR_RADIUS,
        Stroke::new(THIN, chrome.text.gamma_multiply(0.6)),
    );
    let rim = match view.attached {
        true => Stroke::new(ATTACHED_RIM_WIDTH, chrome.accent),
        false => Stroke::new(RIM_WIDTH, chrome.border),
    };
    painter.circle_stroke(center, RADIUS, rim);
    if let Some(progress) = view.progress.filter(|&progress| progress > 0.0) {
        let stroke = Stroke::new(ATTACHED_RIM_WIDTH, chrome.accent);
        painter.add(egui::Shape::line(arc(center, RADIUS, progress), stroke));
    }
    if view.flash > 0.0 {
        let stroke = Stroke::new(ATTACHED_RIM_WIDTH, chrome.accent.gamma_multiply(view.flash));
        painter.circle_stroke(center, RADIUS, stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 760.0))
    }

    #[test]
    fn the_loupe_starts_in_the_upper_corner_away_from_the_finger() {
        let left_finger = Pos2::new(100.0, 400.0);
        let right_finger = Pos2::new(300.0, 400.0);
        assert_eq!(
            LoupeCorner::next(None, left_finger, phone()),
            LoupeCorner::Right
        );
        assert_eq!(
            LoupeCorner::next(None, right_finger, phone()),
            LoupeCorner::Left
        );
    }

    #[test]
    fn the_hold_arc_runs_clockwise_from_the_top() {
        let center = Pos2::new(100.0, 100.0);
        let ends = |fraction| {
            let points = arc(center, 10.0, fraction);
            (points[0], points[points.len() - 1])
        };
        let (start, quarter) = ends(0.25);
        assert!(start.distance(Pos2::new(100.0, 90.0)) < 1e-3);
        assert!(quarter.distance(Pos2::new(110.0, 100.0)) < 1e-3);
        let (_, half) = ends(0.5);
        assert!(half.distance(Pos2::new(100.0, 110.0)) < 1e-3);
    }

    #[test]
    fn the_loupe_stays_put_until_the_finger_comes_near_it() {
        let crossing = Pos2::new(300.0, 400.0);
        assert_eq!(
            LoupeCorner::next(Some(LoupeCorner::Right), crossing, phone()),
            LoupeCorner::Right
        );
        let under_it = LoupeCorner::Right.center(phone()) + Vec2::new(0.0, RADIUS);
        assert_eq!(
            LoupeCorner::next(Some(LoupeCorner::Right), under_it, phone()),
            LoupeCorner::Left
        );
    }
}
