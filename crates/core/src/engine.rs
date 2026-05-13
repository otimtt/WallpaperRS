use crate::config::{AppConfig, WallpaperKind};
use crate::renderer::Renderer;
use anyhow::Result;
use std::time::Instant;

pub struct WallpaperEngine {
    pub config: AppConfig,
    renderer: Option<Box<dyn Renderer>>,
    last_frame: Instant,
    frame_time_accum: f32,
    pub gpu_usage: f32,
    pub fps_actual: f32,
    pub paused: bool,
}

impl WallpaperEngine {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            renderer: None,
            last_frame: Instant::now(),
            frame_time_accum: 0.0,
            gpu_usage: 0.0,
            fps_actual: 0.0,
            paused: false,
        }
    }

    pub fn set_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderer = Some(renderer);
    }

    pub fn tick(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Result<()> {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_time_accum += delta;
        if self.frame_time_accum >= 1.0 {
            self.fps_actual = 1.0 / delta;
            self.frame_time_accum = 0.0;
        }

        if self.paused {
            return Ok(());
        }

        if let Some(renderer) = &mut self.renderer {
            renderer.update(delta);
            renderer.render(device, queue, view)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    pub fn renderer_name(&self) -> &str {
        self.renderer
            .as_ref()
            .map(|r| r.name())
            .unwrap_or("Nenhum")
    }

    pub fn wallpaper_kind_label(&self) -> &str {
        self.config
            .active_wallpaper
            .as_ref()
            .map(|w| w.kind.label())
            .unwrap_or("-")
    }
}
