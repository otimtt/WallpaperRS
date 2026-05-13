use crate::state::WallpaperState;
use anyhow::Result;
use std::{sync::Arc, thread::JoinHandle, time::{Duration, Instant}};
use wallpaper_core::{
    config::{AppConfig, WallpaperEntry, WallpaperKind},
    engine::WallpaperEngine,
    renderer::{RenderContext, Renderer},
};

pub struct WallpaperWindow {
    thread: Option<JoinHandle<()>>,
    pub state: Arc<WallpaperState>,
}

impl WallpaperWindow {
    pub fn spawn(entry: WallpaperEntry, config: AppConfig) -> Self {
        let state = WallpaperState::new(config.performance.target_fps);
        let state2 = Arc::clone(&state);

        let thread = std::thread::spawn(move || {
            if let Err(e) = run_wallpaper(entry, config, state2) {
                log::error!("Wallpaper thread: {e}");
            }
        });

        Self { thread: Some(thread), state }
    }

    pub fn stop(&mut self) {
        self.state.stop();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }

    pub fn toggle_pause(&self) {
        if self.state.is_paused() { self.state.resume(); }
        else { self.state.pause(); }
    }

    pub fn is_paused(&self) -> bool { self.state.is_paused() }
}

impl Drop for WallpaperWindow {
    fn drop(&mut self) { self.stop(); }
}

// ── Render thread entry point ─────────────────────────────────────────────────

fn run_wallpaper(
    entry: WallpaperEntry,
    config: AppConfig,
    state: Arc<WallpaperState>,
) -> Result<()> {
    // On Windows: create a Win32 desktop window and inject into WorkerW.
    // On other OS: abort gracefully (only targeting Windows).
    #[cfg(not(target_os = "windows"))]
    {
        log::warn!("Wallpaper rendering only works on Windows. Skipping.");
        while !state.is_stopped() { std::thread::sleep(Duration::from_millis(200)); }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    run_wallpaper_win(entry, config, state)
}

#[cfg(target_os = "windows")]
fn run_wallpaper_win(
    entry: WallpaperEntry,
    config: AppConfig,
    state: Arc<WallpaperState>,
) -> Result<()> {
    use raw_window_handle::{
        RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    };
    use std::num::NonZeroIsize;

    // 1. Create desktop window
    let (hwnd_raw, w, h) = wallpaper_platform_win::create_wallpaper_window()?;

    // 2. Build WGPU instance (prefer DX12 on Windows)
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
        ..Default::default()
    });

    // 3. Create surface from HWND
    let surface: wgpu::Surface<'static> = unsafe {
        let raw_win = Win32WindowHandle::new(
            NonZeroIsize::new(hwnd_raw as isize)
                .ok_or_else(|| anyhow::anyhow!("Invalid HWND"))?,
        );
        let raw_dpy = WindowsDisplayHandle::new();
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: RawDisplayHandle::Windows(raw_dpy),
            raw_window_handle:  RawWindowHandle::Win32(raw_win),
        })?
    };

    // 4. Request adapter + device
    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let ctx = rt.block_on(RenderContext::new(&instance, &surface, w, h))?;

    // 5. Configure surface
    let surface_cfg = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: ctx.surface_format,
        width: w,
        height: h,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&ctx.device, &surface_cfg);

    // 6. Build the right renderer
    let renderer: Box<dyn Renderer> = build_renderer(&entry, &ctx, hwnd_raw, w, h)?;
    let mut engine = WallpaperEngine::new(config.clone());
    engine.set_renderer(renderer);

    // 7. Render loop
    let pause_on_fs   = config.performance.pause_on_fullscreen;
    let mut last_full = Instant::now();
    let mut last_frame = Instant::now();

    loop {
        wallpaper_platform_win::pump_messages(hwnd_raw);

        if state.is_stopped() { break; }

        // Fullscreen check (every ~1 s to not spam Win32)
        if pause_on_fs && last_full.elapsed() >= Duration::from_secs(1) {
            last_full = Instant::now();
            engine.paused = wallpaper_platform_win::is_fullscreen_running();
        }

        if state.is_paused() {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        // Render
        match surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                if let Err(e) = engine.tick(&ctx.device, &ctx.queue, &view) {
                    log::warn!("Engine tick error: {e}");
                }
                frame.present();
            }
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                surface.configure(&ctx.device, &surface_cfg);
            }
            Err(e) => log::warn!("Surface: {e}"),
        }

        // Frame rate cap
        let target = Duration::from_secs_f64(1.0 / state.fps() as f64);
        let elapsed = last_frame.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
        last_frame = Instant::now();
    }

    wallpaper_platform_win::destroy_window(hwnd_raw);
    Ok(())
}

// ── Renderer factory ──────────────────────────────────────────────────────────

fn build_renderer(
    entry: &WallpaperEntry,
    ctx: &RenderContext,
    hwnd_raw: usize,
    w: u32,
    h: u32,
) -> Result<Box<dyn Renderer>> {
    use wallpaper_renderer_scene::SceneRenderer;
    use wallpaper_renderer_shader::ShaderRenderer;
    use wallpaper_renderer_video::VideoRenderer;
    use wallpaper_renderer_web::WebRenderer;

    match entry.kind {
        WallpaperKind::Shader => {
            let src = if entry.path.exists() {
                std::fs::read_to_string(&entry.path)?
            } else {
                include_str!("../../../assets/shaders/default.wgsl").to_string()
            };
            Ok(Box::new(ShaderRenderer::new(&ctx.device, ctx.surface_format, &src, w, h)?))
        }
        WallpaperKind::Video => {
            Ok(Box::new(VideoRenderer::new(&ctx.device, ctx.surface_format, entry.path.clone(), w, h)?))
        }
        WallpaperKind::Scene => {
            Ok(Box::new(SceneRenderer::new(&ctx.device, ctx.surface_format, w, h)?))
        }
        WallpaperKind::Web => {
            Ok(Box::new(WebRenderer::new(hwnd_raw, entry.path.to_string_lossy().into(), w, h)?))
        }
    }
}

// Type aliases to keep imports clean
use wallpaper_platform_win as wallpaper_platform_win;
use wallpaper_renderer_scene;
use wallpaper_renderer_shader;
use wallpaper_renderer_video;
use wallpaper_renderer_web;
