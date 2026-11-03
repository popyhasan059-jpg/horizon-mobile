pub const SHADER_SOURCE: &str = include_str!("shader.wgsl");

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub fn create_shader_module(&self) -> wgpu::ShaderModule {
        self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Horizon Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    SHADER_SOURCE.into()
                ),
            }
        )
    }

    pub async fn new() -> Result<Self, String> {
        let instance =
            wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                ..Default::default()
            })
            .await
            .map_err(|e| format!("GPU adapter error: {e}"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| format!("GPU device error: {e}"))?;

        Ok(Self { device, queue })
    }
}
