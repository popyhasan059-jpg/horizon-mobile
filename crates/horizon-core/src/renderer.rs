use wgpu::util::DeviceExt;

pub const SHADER_SOURCE: &str = include_str!("shader.wgsl");

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] =
    wgpu::vertex_attr_array![0 => Float32x3];

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
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

    pub fn create_depth_texture(
        &self,
        width: u32,
        height: u32,
    ) -> DepthTexture {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Horizon Depth Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        );

        let view = texture.create_view(
            &wgpu::TextureViewDescriptor::default(),
        );

        DepthTexture { texture, view }
    }

    pub fn create_gpu_mesh(
        &self,
        mesh: &crate::mesh::Mesh,
    ) -> GpuMesh {
        let mut vertex_bytes = Vec::new();

        for vertex in &mesh.vertices {
            for component in vertex {
                vertex_bytes.extend_from_slice(
                    &component.to_ne_bytes(),
                );
            }
        }

        let mut index_bytes = Vec::new();

        for index in &mesh.indices {
            index_bytes.extend_from_slice(
                &index.to_ne_bytes(),
            );
        }

        let vertex_buffer =
            self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Horizon Cube Vertices"),
                    contents: &vertex_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );

        let index_buffer =
            self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Horizon Cube Indices"),
                    contents: &index_bytes,
                    usage: wgpu::BufferUsages::INDEX,
                },
            );

        GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
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
                depth_stencil: Some(
                    wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    },
                ),
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

    pub fn encode_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        mesh: &GpuMesh,
    ) {
        let color_attachments = [
            Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(
                        wgpu::Color::BLACK,
                    ),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ];

        let mut pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Horizon Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    },
                ),
                timestamp_writes: None,
                occlusion_query_set: None,
            },
        );

        pass.set_pipeline(pipeline);

        pass.set_vertex_buffer(
            0,
            mesh.vertex_buffer.slice(..),
        );

        pass.set_index_buffer(
            mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        pass.draw_indexed(
            0..mesh.index_count,
            0,
            0..1,
        );
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
