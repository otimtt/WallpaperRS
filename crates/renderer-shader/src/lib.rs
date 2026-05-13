use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wallpaper_core::renderer::Renderer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    _pad: f32,
}

pub struct ShaderRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    time: f32,
    width: u32,
    height: u32,
}

impl ShaderRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        shader_source: &str,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shader BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shader BG"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shader Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            uniform_buf,
            bind_group,
            time: 0.0,
            width,
            height,
        })
    }
}

impl Renderer for ShaderRenderer {
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Result<()> {
        let uniforms = Uniforms {
            time: self.time,
            width: self.width as f32,
            height: self.height as f32,
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shader Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, delta: f32) {
        self.time += delta;
    }

    fn name(&self) -> &str {
        "Shader"
    }
}
