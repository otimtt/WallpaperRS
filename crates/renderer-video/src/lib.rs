use anyhow::Result;
use std::path::PathBuf;
use wallpaper_core::renderer::Renderer;

// ── Shared textured-quad pipeline builder ────────────────────────────────────

fn build_textured_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Video Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../assets/shaders/textured.wgsl").into()
        ),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bgl],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Video Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader, entry_point: "vs_main",
            buffers: &[], compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader, entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
        multisample: Default::default(),
        multiview: None,
        cache: None,
    })
}

fn build_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Video BGL"),
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
        ],
    })
}

fn build_bind_group(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Video BG"),
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
        ],
    })
}

// ── Windows — Windows Media Foundation decoder ────────────────────────────────

#[cfg(target_os = "windows")]
mod mf {
    use anyhow::{anyhow, Result};
    use std::path::PathBuf;
    use windows::{
        core::{GUID, PCWSTR},
        Win32::Media::MediaFoundation::*,
        Win32::System::Com::StructuredStorage::*,
        Win32::System::Variant::VT_I8,
    };

    pub struct MfDecoder {
        reader:       IMFSourceReader,
        pub vw:       u32,
        pub vh:       u32,
        pub fps_num:  u64,
        pub fps_den:  u64,
        path:         Vec<u16>,
    }

    impl MfDecoder {
        pub fn open(path: &PathBuf) -> Result<Self> {
            unsafe {
                MFStartup(MF_SDK_VERSION, MFSTARTUP_NOSOCKET.0)?;

                let path_wide: Vec<u16> = path
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let reader: IMFSourceReader =
                    MFCreateSourceReaderFromURL(PCWSTR(path_wide.as_ptr()), None)?;

                // Force ARGB32 output
                let out_type: IMFMediaType = MFCreateMediaType()?;
                out_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                out_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
                reader.SetCurrentMediaType(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &out_type,
                )?;

                // Disable audio stream to avoid noise
                reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
                reader.SetStreamSelection(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, true)?;

                // Get dimensions & FPS
                let actual: IMFMediaType = reader.GetCurrentMediaType(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                )?;

                let mut vw = 0u32;
                let mut vh = 0u32;
                MFGetAttributeSize(&actual, &MF_MT_FRAME_SIZE, &mut vw, &mut vh)?;

                let mut fps_num = 0u64;
                let mut fps_den = 0u64;
                let _ = MFGetAttributeRatio(&actual, &MF_MT_FRAME_RATE, &mut fps_num, &mut fps_den);
                if fps_den == 0 { fps_num = 30; fps_den = 1; }

                Ok(Self { reader, vw, vh, fps_num, fps_den, path: path_wide })
            }
        }

        /// Read the next ARGB frame. Returns None at EOF (caller should seek/reopen).
        pub fn read_frame(&mut self) -> Result<Option<Vec<u8>>> {
            unsafe {
                let mut flags = 0u32;
                let mut _ts   = 0i64;
                let mut sample: Option<IMFSample> = None;

                self.reader.ReadSample(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut _ts),
                    Some(&mut sample),
                )?;

                if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 != 0 {
                    return Ok(None);
                }

                let sample = match sample {
                    Some(s) => s,
                    None => return Ok(None),
                };

                let buf: IMFMediaBuffer = sample.ConvertToContiguousBuffer()?;
                let mut ptr: *mut u8 = std::ptr::null_mut();
                let mut max = 0u32;
                let mut cur = 0u32;
                buf.Lock(&mut ptr, Some(&mut max), Some(&mut cur))?;
                let frame = std::slice::from_raw_parts(ptr, cur as usize).to_vec();
                buf.Unlock()?;

                Ok(Some(frame))
            }
        }

        /// Seek back to beginning of the video (for looping).
        pub fn seek_start(&mut self) -> Result<()> {
            unsafe {
                // Reopen from path — simpler and more reliable than PROPVARIANT seek
                let reader: IMFSourceReader =
                    MFCreateSourceReaderFromURL(PCWSTR(self.path.as_ptr()), None)?;
                let out_type: IMFMediaType = MFCreateMediaType()?;
                out_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                out_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
                reader.SetCurrentMediaType(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &out_type,
                )?;
                reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
                reader.SetStreamSelection(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, true)?;
                self.reader = reader;
            }
            Ok(())
        }
    }

    impl Drop for MfDecoder {
        fn drop(&mut self) {
            unsafe { let _ = MFShutdown(); }
        }
    }
}

// ── VideoRenderer ─────────────────────────────────────────────────────────────

pub struct VideoRenderer {
    pipeline:      wgpu::RenderPipeline,
    bgl:           wgpu::BindGroupLayout,
    bind_group:    wgpu::BindGroup,
    sampler:       wgpu::Sampler,
    frame_texture: wgpu::Texture,
    pending_frame: Option<Vec<u8>>,
    vw: u32,
    vh: u32,
    width:  u32,
    height: u32,
    time_to_next: f32,
    frame_period:  f32,
    #[cfg(target_os = "windows")]
    decoder: mf::MfDecoder,
    path: PathBuf,
    surface_format: wgpu::TextureFormat,
}

impl VideoRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        path: PathBuf,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        #[cfg(target_os = "windows")]
        let decoder = mf::MfDecoder::open(&path)?;

        let (vw, vh, frame_period) = {
            #[cfg(target_os = "windows")]
            { (decoder.vw, decoder.vh, decoder.fps_den as f32 / decoder.fps_num as f32) }
            #[cfg(not(target_os = "windows"))]
            { (width, height, 1.0 / 30.0) }
        };

        let bgl  = build_bgl(device);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Video Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let frame_texture = Self::make_texture(device, vw, vh);
        let view = frame_texture.create_view(&Default::default());
        let bind_group  = build_bind_group(device, &bgl, &view, &sampler);
        let pipeline     = build_textured_pipeline(device, surface_format, &bgl);

        Ok(Self {
            pipeline, bgl, bind_group, sampler, frame_texture,
            pending_frame: None,
            vw, vh, width, height,
            time_to_next: 0.0,
            frame_period,
            #[cfg(target_os = "windows")]
            decoder,
            path,
            surface_format,
        })
    }

    fn make_texture(device: &wgpu::Device, w: u32, h: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Video Frame"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }
}

impl Renderer for VideoRenderer {
    fn update(&mut self, delta: f32) {
        self.time_to_next -= delta;
        if self.time_to_next > 0.0 {
            return;
        }
        self.time_to_next = self.frame_period;

        #[cfg(target_os = "windows")]
        {
            match self.decoder.read_frame() {
                Ok(Some(frame)) => self.pending_frame = Some(frame),
                Ok(None) => {
                    // Loop: reopen reader from start
                    if let Err(e) = self.decoder.seek_start() {
                        log::warn!("Video seek_start failed: {e}");
                    }
                }
                Err(e) => log::warn!("Video read_frame error: {e}"),
            }
        }
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Result<()> {
        // Upload new frame if available
        if let Some(frame) = self.pending_frame.take() {
            let bytes_per_row = self.vw * 4;
            if frame.len() as u32 >= bytes_per_row * self.vh {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.frame_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &frame,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(self.vh),
                    },
                    wgpu::Extent3d { width: self.vw, height: self.vh, depth_or_array_layers: 1 },
                );
                // Rebuild bind group to point at updated texture
                let tex_view = self.frame_texture.create_view(&Default::default());
                self.bind_group = build_bind_group(device, &self.bgl, &tex_view, &self.sampler);
            }
        }

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Video Pass"),
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

    fn name(&self) -> &str { "Vídeo" }
}
