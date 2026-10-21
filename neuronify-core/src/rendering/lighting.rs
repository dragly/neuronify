//! A look between flat colours, which carry the voltage, and full lighting,
//! which shows the shapes and their shadows.

use glam::Vec3;
use visula::post_process::config::{BloomConfig, Tonemapping};
use visula::Expression;

/// Share of each colour that comes from lighting. The rest is the flat colour.
const SHADING: f32 = 0.5;
/// visula's ambient and fill light are fixed, so a shadow keeps about a fifth of the
/// light. A weaker main light, brightened again through its colour, lets shadows keep
/// almost half of it while lit surfaces stay at their own colour.
const LIGHT_INTENSITY: f32 = 0.25;
const LIGHT_BRIGHTNESS: f32 = 2.3;

/// The colour, partly lit, without a specular highlight that would read as a voltage.
pub fn shaded(color: Expression) -> Expression {
    color.clone() * (1.0 - SHADING) + color.directional_lit() * SHADING
}

pub fn configure_lighting(application: &mut visula::Application) {
    application.light.direction = Vec3::new(-0.3, -1.0, -0.2).normalize();
    application.light.intensity = LIGHT_INTENSITY;
    application.light.color = Vec3::splat(LIGHT_BRIGHTNESS);
    application.light.shadow_distance = 300.0;
    // Colours stay as they are, since they show the voltage
    application.post_processor.config.tonemapping = Tonemapping::None;
    // Only flashes are brighter than white, so only they bloom
    application.post_processor.config.bloom = Some(BloomConfig {
        threshold: 1.1,
        intensity: 2.5,
        mip_levels: 5,
    });
    application.post_processor.enable_bloom(
        &application.device,
        application.config.width,
        application.config.height,
    );
    // visula's SSAO expects a multisampled depth buffer, which the web build does not have
    #[cfg(not(target_arch = "wasm32"))]
    {
        application.post_processor.config.ssao = Some(visula::post_process::config::SsaoConfig {
            radius: 2.0,
            bias: 0.02,
            intensity: 0.75,
        });
        application.post_processor.enable_ssao(
            &application.device,
            &application.queue,
            application.config.width,
            application.config.height,
            &application.camera,
            &application.depth_texture,
        );
    }
}
