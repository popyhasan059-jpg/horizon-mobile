pub const SHADER_SOURCE: &str = include_str!("shader.wgsl");

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] =
    wgpu::vertex_attr_array![0 => Float32x3];

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
                    SHADER_SOURCE.into(),
                ),
            },
        )
    }

    pub fn create_render_pipeline(
        &self,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = self.create_shader_module();

        let pipeline_layout =
            self.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Horizon Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                },
            );

        let vertex_buffers = [
            wgpu::VertexBufferLayout {
                array_stride: 12,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &VERTEX_ATTRIBUTES,
            },
        ];

        let color_targets = [
            Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
        ];

        self.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Horizon 3D Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &color_targets,
                }),
                multiview: None,
                cache: None,
            },
        )
    }

    pub async fn new() -> Result<Self, String> {
        let instance =
            wgpu::Instance::new(
                &wgpu::InstanceDescriptor::default(),
            );

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference:
                        wgpu::PowerPreference::LowPower,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| format!("GPU adapter error: {e}"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| format!("GPU device error: {e}"))?;

        Ok(Self { device, queue })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_is_valid() {
        let module =
            naga::front::wgsl::parse_str(SHADER_SOURCE)
                .expect("Horizon shader syntax should be valid");

        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("Horizon shader validation failed");
    }
}
