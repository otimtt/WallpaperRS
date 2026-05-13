use anyhow::Result;

pub trait Renderer: Send + Sync {
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Result<()>;
    fn resize(&mut self, width: u32, height: u32);
    fn update(&mut self, delta: f32);
    fn name(&self) -> &str;
}

pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl RenderContext {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'_>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No compatible GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WallpaperRS Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        Ok(Self {
            device,
            queue,
            surface_format,
            width,
            height,
        })
    }
}
