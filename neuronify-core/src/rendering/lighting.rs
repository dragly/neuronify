//! A look between flat colours, which carry the voltage, and full lighting,
//! which shows the shapes and their shadows.

use glam::Vec3;
use visula::post_process::config::{BloomConfig, SsaoConfig, Tonemapping};
use visula::Expression;

use crate::rendering::colors::Theme;

/// Share of each colour that comes from lighting. The rest is the flat colour.
const SHADING: f32 = 0.5;
pub const BLOOM_THRESHOLD: f32 = 1.1;

/// The colour, partly lit, without a specular highlight that would read as a voltage.
pub fn shaded(color: Expression) -> Expression {
    color.clone() * (1.0 - SHADING) + color.directional_lit() * SHADING
}

pub fn configure_lighting(application: &mut visula::Application, theme: Theme) {
    application.light.direction = Vec3::new(-0.3, -1.0, -0.2).normalize();
    application.light.shadow_distance = 300.0;
    // Colours stay as they are, since they show the voltage
    application.post_processor.config.tonemapping = Tonemapping::None;
    set_theme_lighting(application, theme);
    application.post_processor.config.bloom = Some(BloomConfig {
        threshold: BLOOM_THRESHOLD,
        intensity: 1.75,
        mip_levels: 5,
    });
    application.post_processor.enable_bloom(
        &application.device,
        application.config.width,
        application.config.height,
    );
    // visula's SSAO expects a multisampled depth buffer, which the web build does not have
    #[cfg(not(target_arch = "wasm32"))]
    application.post_processor.enable_ssao(
        &application.device,
        &application.queue,
        application.config.width,
        application.config.height,
        &application.camera,
        &application.depth_texture,
    );
}

/// visula's ambient and fill light are fixed, so a shadow keeps about a fifth of the
/// light. A weaker main light, brightened again through its colour, lets shadows keep
/// more of it while lit surfaces stay at their own colour. A light floor takes darker
/// shadows than a dark one before they look heavy.
pub fn set_theme_lighting(application: &mut visula::Application, theme: Theme) {
    let (intensity, brightness, occlusion) = match theme {
        Theme::Dark => (0.25, 2.3, 0.75),
        Theme::Light => (0.35, 1.9, 1.0),
    };
    application.light.intensity = intensity;
    application.light.color = Vec3::splat(brightness);
    application.post_processor.config.ssao = Some(SsaoConfig {
        radius: 2.0,
        bias: 0.02,
        intensity: occlusion,
    });
}
