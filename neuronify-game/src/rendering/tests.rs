/// Headless GPU tests for rendering pipeline creation.
#[cfg(test)]
mod tests {
    use visula::{Camera, DirectionalLight, InstanceBuffer, RenderingDescriptor};

    async fn try_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok()?;
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .ok()
    }

    /// Verify that the dendrite Cylinders pipeline compiles without shader errors.
    #[test]
    fn test_dendrite_pipeline_compiles() {
        let Some((device, _queue)) = pollster::block_on(try_headless_device()) else {
            eprintln!("No wgpu adapter available – skipping test_dendrite_pipeline_compiles");
            return;
        };
        let camera = Camera::new(&device);
        let light = DirectionalLight::new(&device);
        let desc = RenderingDescriptor {
            device: &device,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            camera: &camera,
            light: &light,
            sample_count: 1,
        };
        use visula::InstanceBuffer;
        let cylinder_buffer = InstanceBuffer::<crate::rendering::CylinderData>::new(&device);
        crate::rendering::create_dendrite_pipeline(&desc, &cylinder_buffer)
            .expect("dendrite pipeline should compile without errors");
    }
}
