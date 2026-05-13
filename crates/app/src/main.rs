mod icon;
mod state;
mod tray;
mod ui;
mod wallpaper_window;

use eframe::egui;
use tray::{AppTray, TrayAction};
use ui::{AppUi, UiAction};
use wallpaper_window::WallpaperWindow;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (tray_rgba, tw, th) = icon::rgba_32();
    let (win_rgba,  ww, wh) = icon::rgba_256();

    let tray = AppTray::new(tray_rgba, tw, th)
        .map_err(|e| log::warn!("Tray unavailable: {e}"))
        .ok();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Wallpaper RS")
            .with_inner_size([1100.0, 680.0])
            .with_min_inner_size([800.0, 520.0])
            .with_icon(egui::IconData { rgba: win_rgba, width: ww, height: wh })
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native("Wallpaper RS", options, Box::new(move |cc| {
        cc.egui_ctx.set_pixels_per_point(1.0);
        let mut app = App::new(tray);
        app.ui.populate_demo_wallpapers();
        Ok(Box::new(app))
    }))
}

struct App {
    ui:               AppUi,
    tray:             Option<AppTray>,
    wallpaper:        Option<WallpaperWindow>,
    frames_since_upd: u32,
    last_fps_update:  std::time::Instant,
}

impl App {
    fn new(tray: Option<AppTray>) -> Self {
        Self {
            ui:               AppUi::default(),
            tray,
            wallpaper:        None,
            frames_since_upd: 0,
            last_fps_update:  std::time::Instant::now(),
        }
    }

    fn apply_wallpaper(&mut self, entry: wallpaper_core::config::WallpaperEntry) {
        // Stop existing wallpaper first
        if let Some(mut w) = self.wallpaper.take() { w.stop(); }

        log::info!("Applying wallpaper: {} ({:?})", entry.name, entry.kind);
        let config = self.ui.config.clone();
        self.wallpaper = Some(WallpaperWindow::spawn(entry, config));
        self.ui.wallpaper_active = true;
        self.ui.wallpaper_paused = false;
    }

    fn stop_wallpaper(&mut self) {
        if let Some(mut w) = self.wallpaper.take() { w.stop(); }
        self.ui.wallpaper_active = false;
        self.ui.wallpaper_paused = false;
    }

    fn toggle_pause(&mut self) {
        if let Some(w) = &self.wallpaper {
            w.toggle_pause();
            self.ui.wallpaper_paused = w.is_paused();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // FPS counter
        self.frames_since_upd += 1;
        let elapsed = self.last_fps_update.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            self.ui.fps = self.frames_since_upd as f32 / elapsed;
            self.frames_since_upd = 0;
            self.last_fps_update = std::time::Instant::now();
        }

        // Tray icon events
        if let Some(tray) = &self.tray {
            match tray.poll() {
                Some(TrayAction::Quit)  => { ctx.send_viewport_cmd(egui::ViewportCommand::Close); return; }
                Some(TrayAction::Show)  => { ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true)); ctx.send_viewport_cmd(egui::ViewportCommand::Focus); }
                Some(TrayAction::Pause) => self.toggle_pause(),
                None => {}
            }
        }

        // UI actions (Apply / Pause / Stop)
        if let Some(action) = self.ui.show(ctx) {
            match action {
                UiAction::Apply(entry) => self.apply_wallpaper(entry),
                UiAction::Pause        => self.toggle_pause(),
                UiAction::Stop         => self.stop_wallpaper(),
            }
        }

        ctx.request_repaint();
    }

    fn on_exit(&mut self) {
        self.stop_wallpaper();
        log::info!("Wallpaper RS encerrado.");
    }
}
