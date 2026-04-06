/// Headless GPU tests for rendering pipeline creation.
///
/// These tests catch naga shader-validation errors (wrong struct_span, bad FunctionArgument
/// indices, type mismatches, etc.) that only appear at pipeline-creation time, not during
/// the simulation loop.  They run with a real wgpu device backed by whatever adapter the
/// host provides (software fallback included); if no adapter is available at all the tests
/// are skipped gracefully.
#[cfg(test)]
mod tests {
    use visula::{Camera, DirectionalLight, InstanceBuffer, RenderingDescriptor, UniformBuffer};

    use crate::rendering::blood_vessels::{
        BloodParticle, VesselTime, create_particle_pipeline, create_vessel_pipeline,
    };

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

    /// Verify that the blood-vessel mesh pipeline compiles without shader errors.
    #[test]
    fn test_vessel_pipeline_compiles() {
        let Some((device, _queue)) = pollster::block_on(try_headless_device()) else {
            eprintln!("No wgpu adapter available – skipping test_vessel_pipeline_compiles");
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
        create_vessel_pipeline(&desc).expect("vessel pipeline should compile without errors");
    }

    /// Verify that the blood-particle Spheres pipeline compiles without shader errors.
    ///
    /// This catches the class of error where instance-buffer fields are accidentally used
    /// in the fragment stage (SphereMaterial), causing naga to generate a
    /// FunctionArgument index that exceeds the function's argument count.
    #[test]
    fn test_particle_pipeline_compiles() {
        let Some((device, _queue)) = pollster::block_on(try_headless_device()) else {
            eprintln!("No wgpu adapter available – skipping test_particle_pipeline_compiles");
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
        let particle_buffer = InstanceBuffer::<BloodParticle>::new(&device);
        let time_buffer = UniformBuffer::<VesselTime>::new(&device);
        create_particle_pipeline(&desc, &particle_buffer, &time_buffer)
            .expect("particle pipeline should compile without errors");
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
