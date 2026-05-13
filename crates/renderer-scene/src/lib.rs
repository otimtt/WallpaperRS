use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wallpaper_core::renderer::Renderer;

const MAX_PARTICLES: usize = 8_000;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadVert {
    local_pos: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ParticleInstance {
    pos:        [f32; 2],
    life_ratio: f32,
    size:       f32,
    heat:       f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    aspect: f32,
    _pad: [f32; 3],
}

struct Particle {
    x:   f32,
    y:   f32,
    vx:  f32,
    vy:  f32,
    life:     f32,
    max_life: f32,
    size: f32,
}

impl Particle {
    fn dead() -> Self {
        Self { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, life: 0.0, max_life: 1.0, size: 1.0 }
    }

    fn respawn(&mut self, seed: f32) {
        let angle = fast_rand(seed) * std::f32::consts::TAU;
        let r     = fast_rand(seed + 1.0) * 0.2;
        self.x    = angle.cos() * r;
        self.y    = angle.sin() * r;
        let speed = 0.04 + fast_rand(seed + 2.0) * 0.18;
        self.vx   = angle.cos() * speed;
        self.vy   = angle.sin() * speed;
        self.max_life = 1.5 + fast_rand(seed + 3.0) * 3.0;
        self.life     = self.max_life;
        self.size     = 2.0 + fast_rand(seed + 4.0) * 5.0;
    }
}

fn fast_rand(seed: f32) -> f32 {
    (seed * 1_234.567_f32).sin().abs().fract()
}

pub struct SceneRenderer {
    pipeline:      wgpu::RenderPipeline,
    quad_vbuf:     wgpu::Buffer,
    inst_vbuf:     wgpu::Buffer,
    uniform_buf:   wgpu::Buffer,
    bind_group:    wgpu::BindGroup,
    particles:     Vec<Particle>,
    instances:     Vec<ParticleInstance>,
    width:  u32,
    height: u32,
    time:   f32,
}

impl SceneRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let shader_src = include_str!("../../../assets/shaders/particles.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particles"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Uniforms"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene BG"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Scene Layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let quad_verts: &[QuadVert] = &[
            QuadVert { local_pos: [-1.0, -1.0] },
            QuadVert { local_pos: [ 1.0, -1.0] },
            QuadVert { local_pos: [-1.0,  1.0] },
            QuadVert { local_pos: [ 1.0,  1.0] },
        ];

        let quad_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad VBuf"),
            contents: bytemuck::cast_slice(quad_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let inst_vbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance VBuf"),
            size: (std::mem::size_of::<ParticleInstance>() * MAX_PARTICLES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<QuadVert>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ParticleInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x2,
                            2 => Float32,
                            3 => Float32,
                            4 => Float32,
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
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

        let mut particles: Vec<Particle> = (0..MAX_PARTICLES).map(|_| Particle::dead()).collect();
        for (i, p) in particles.iter_mut().enumerate() {
            p.respawn(i as f32 * 0.618);
            p.life = p.max_life * fast_rand(i as f32 * 1.414);
        }

        Ok(Self {
            pipeline,
            quad_vbuf,
            inst_vbuf,
            uniform_buf,
            bind_group,
            particles,
            instances: Vec::with_capacity(MAX_PARTICLES),
            width,
            height,
            time: 0.0,
        })
    }
}

impl Renderer for SceneRenderer {
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Result<()> {
        let aspect = self.width as f32 / self.height as f32;
        let uniforms = SceneUniforms { aspect, _pad: [0.0; 3] };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        let inst_bytes = bytemuck::cast_slice(&self.instances);
        queue.write_buffer(&self.inst_vbuf, 0, inst_bytes);

        let count = self.instances.len() as u32;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            if count > 0 {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_vbuf.slice(..));
                pass.set_vertex_buffer(1, self.inst_vbuf.slice(..));
                pass.draw(0..4, 0..count);
            }
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
        let capped = delta.min(0.05);

        self.instances.clear();
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.life -= capped;
            if p.life <= 0.0 {
                p.respawn(i as f32 * 1.618 + self.time * 0.1);
                continue;
            }
            p.x  += p.vx * capped;
            p.y  += p.vy * capped;
            p.vx *= 1.0 - capped * 0.4;
            p.vy *= 1.0 - capped * 0.4;
            p.vy += 0.015 * capped;

            let life_ratio = p.life / p.max_life;
            let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
            self.instances.push(ParticleInstance {
                pos: [p.x, p.y],
                life_ratio,
                size: p.size,
                heat: (speed * 12.0).min(1.0),
            });
        }
    }

    fn name(&self) -> &str {
        "Cena 2D"
    }
}

// Needed for buffer_init
use wgpu::util::DeviceExt;
