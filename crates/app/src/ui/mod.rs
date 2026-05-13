pub mod library;
pub mod preview;
pub mod settings;
pub mod theme;

use egui::{Color32, RichText, Stroke, Vec2};
use wallpaper_core::config::{AppConfig, WallpaperEntry};
use library::LibraryPanel;
use preview::{PreviewAction, PreviewPanel};
use settings::SettingsPanel;

#[derive(PartialEq, Clone)]
pub enum NavPage { Library, Settings, About }

pub enum UiAction {
    Apply(WallpaperEntry),
    Pause,
    Stop,
}

pub struct AppUi {
    pub page:             NavPage,
    pub library:          LibraryPanel,
    pub config:           AppConfig,
    pub fps:              f32,
    pub gpu_pct:          f32,
    pub wallpaper_active: bool,
    pub wallpaper_paused: bool,
}

impl Default for AppUi {
    fn default() -> Self {
        Self {
            page:             NavPage::Library,
            library:          LibraryPanel::default(),
            config:           AppConfig::default(),
            fps:              0.0,
            gpu_pct:          0.0,
            wallpaper_active: false,
            wallpaper_paused: false,
        }
    }
}

impl AppUi {
    pub fn populate_demo_wallpapers(&mut self) {
        use wallpaper_core::config::WallpaperKind;
        self.library.wallpapers = vec![
            WallpaperEntry { id: "demo_shader_1".into(), name: "Nebula Crimson".into(),
                author: "WallpaperRS".into(), path: "assets/shaders/default.wgsl".into(),
                kind: WallpaperKind::Shader, thumbnail: None, tags: vec![] },
            WallpaperEntry { id: "demo_shader_2".into(), name: "Dark Pulse".into(),
                author: "WallpaperRS".into(), path: "assets/shaders/default.wgsl".into(),
                kind: WallpaperKind::Shader, thumbnail: None, tags: vec![] },
            WallpaperEntry { id: "demo_scene_1".into(), name: "Particle Storm".into(),
                author: "WallpaperRS".into(), path: "".into(),
                kind: WallpaperKind::Scene, thumbnail: None, tags: vec![] },
            WallpaperEntry { id: "demo_video_1".into(), name: "City Rain Loop".into(),
                author: "Demo".into(), path: "wallpapers/city_rain.mp4".into(),
                kind: WallpaperKind::Video, thumbnail: None, tags: vec![] },
            WallpaperEntry { id: "demo_web_1".into(), name: "Matrix Rain".into(),
                author: "Web".into(), path: "https://example.com/matrix".into(),
                kind: WallpaperKind::Web, thumbnail: None, tags: vec![] },
        ];
    }

    /// Returns an action if the user clicked something that the App needs to handle.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;
        theme::apply(ctx);
        self.top_panel(ctx);
        self.sidebar(ctx);
        action = self.main_content(ctx);
        self.status_bar(ctx);
        action
    }

    fn top_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(48.0)
            .frame(egui::Frame::default().fill(theme::BG_SIDEBAR)
                .inner_margin(egui::Margin::symmetric(16.0, 0.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
                    if ui.is_rect_visible(r) {
                        ui.painter().rect_filled(r, egui::Rounding::same(4.0), theme::ACCENT);
                        ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "W",
                            egui::FontId::proportional(20.0), Color32::WHITE);
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new("WALLPAPER RS").size(15.0).color(theme::TEXT).strong());
                    ui.label(RichText::new("v0.1.0").size(10.0).color(theme::TEXT_FAINT));
                });
            });
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .exact_width(160.0).resizable(false)
            .frame(egui::Frame::default().fill(theme::BG_SIDEBAR)
                .inner_margin(egui::Margin::symmetric(10.0, 16.0)))
            .show(ctx, |ui| {
                self.nav_item(ui, NavPage::Library,  "  Biblioteca");
                ui.add_space(2.0);
                self.nav_item(ui, NavPage::Settings, "  Configurações");
                ui.add_space(2.0);
                self.nav_item(ui, NavPage::About,    "  Sobre");

                let rem = ui.available_height() - 40.0;
                if rem > 0.0 { ui.add_space(rem); }

                if self.wallpaper_active {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::hover());
                    if ui.is_rect_visible(r) {
                        let p = ui.painter();
                        p.rect_filled(r, egui::Rounding::same(4.0), theme::BG_ACTIVE);
                        p.rect_stroke(r, egui::Rounding::same(4.0), Stroke::new(1.0, theme::ACCENT));
                        let label = if self.wallpaper_paused { "⏸ PAUSADO" } else { "● ATIVO" };
                        let col   = if self.wallpaper_paused { theme::TEXT_DIM } else { theme::ACCENT };
                        p.text(r.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(9.0), col);
                    }
                }
            });
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, page: NavPage, label: &str) {
        let sel = self.page == page;
        let (fill, col, stroke) = if sel {
            (theme::BG_ACTIVE, theme::ACCENT_BRIGHT, Stroke::new(1.0, theme::ACCENT))
        } else {
            (Color32::TRANSPARENT, theme::TEXT_DIM, Stroke::NONE)
        };
        if ui.add(egui::Button::new(RichText::new(label).color(col).size(13.0))
            .fill(fill).stroke(stroke).min_size(Vec2::new(140.0, 32.0))).clicked()
        {
            self.page = page;
        }
    }

    fn main_content(&mut self, ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG).inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                match self.page {
                    NavPage::Library => {
                        let selected = self.library.wallpapers.iter()
                            .find(|w| self.library.selected.as_deref() == Some(&w.id))
                            .cloned();

                        // Preview + controls
                        let preview_action = PreviewPanel::show(
                            ui,
                            selected.as_ref(),
                            self.fps,
                            self.wallpaper_active,
                            self.wallpaper_paused,
                        );

                        match preview_action {
                            Some(PreviewAction::Apply) => {
                                if let Some(entry) = selected {
                                    action = Some(UiAction::Apply(entry));
                                }
                            }
                            Some(PreviewAction::Pause) => {
                                action = Some(UiAction::Pause);
                            }
                            Some(PreviewAction::Remove) => {
                                action = Some(UiAction::Stop);
                                self.wallpaper_active = false;
                                self.wallpaper_paused = false;
                            }
                            None => {}
                        }

                        ui.add_space(8.0);
                        theme::separator(ui);
                        ui.add_space(4.0);

                        // Library grid — double-click also applies
                        if let Some(activated) = self.library.show(ui) {
                            action = Some(UiAction::Apply(activated));
                        }
                    }
                    NavPage::Settings => {
                        ui.label(RichText::new("Configurações").size(16.0).color(theme::TEXT).strong());
                        ui.add_space(8.0);
                        theme::separator(ui);
                        ui.add_space(8.0);
                        SettingsPanel::show(ui, &mut self.config);
                    }
                    NavPage::About => self.show_about(ui),
                }
            });
        action
    }

    fn status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(30.0)
            .frame(egui::Frame::default().fill(theme::STATUS_BAR)
                .inner_margin(egui::Margin::symmetric(16.0, 0.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let name = self.library.wallpapers.iter()
                        .find(|w| self.library.selected.as_deref() == Some(&w.id))
                        .map(|w| w.name.as_str()).unwrap_or("Nenhum");
                    let col = if self.wallpaper_active { theme::ACCENT } else { theme::TEXT_FAINT };
                    ui.label(RichText::new(if self.wallpaper_active { "●" } else { "○" }).color(col).size(10.0));
                    ui.label(RichText::new(name).color(theme::TEXT_DIM).size(11.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("GPU {:.0}%", self.gpu_pct)).color(theme::TEXT_FAINT).size(10.0).monospace());
                        ui.add_space(12.0);
                        ui.label(RichText::new(format!("{:.0} FPS", self.fps)).color(theme::ACCENT_DIM).size(10.0).monospace());
                        ui.add_space(12.0);
                        ui.label(RichText::new(format!("meta {}fps", self.config.performance.target_fps)).color(theme::TEXT_FAINT).size(10.0));
                    });
                });
            });
    }

    fn show_about(&self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(64.0), egui::Sense::hover());
            if ui.is_rect_visible(r) {
                ui.painter().rect_filled(r, egui::Rounding::same(12.0), theme::ACCENT);
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "W",
                    egui::FontId::proportional(40.0), Color32::WHITE);
            }
            ui.add_space(14.0);
            ui.label(RichText::new("WALLPAPER RS").size(22.0).color(theme::TEXT).strong());
            ui.label(RichText::new("v0.1.0").size(12.0).color(theme::TEXT_FAINT));
            ui.add_space(10.0);
            ui.label(RichText::new("Motor de wallpapers animados de alto desempenho").size(13.0).color(theme::TEXT_DIM));
            ui.label(RichText::new("Rust + WGPU + DX12").size(12.0).color(theme::ACCENT_DIM));
            ui.add_space(20.0);
            theme::separator(ui);
            ui.add_space(12.0);
            for (icon, desc) in &[
                ("⬡", "Shaders WGSL em tempo real"),
                ("▶", "Vídeos MP4 / WebM via Windows Media Foundation"),
                ("✦", "Cenas 2D com 8.000 partículas GPU"),
                ("◈", "Wallpapers HTML/JS via WebView2"),
                ("⚡", "Pausa automática em jogos fullscreen"),
                ("🎛", "Controle fino de FPS e qualidade GPU"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*icon).color(theme::ACCENT).size(14.0));
                    ui.add_space(6.0);
                    ui.label(RichText::new(*desc).color(theme::TEXT_DIM).size(12.0));
                });
                ui.add_space(4.0);
            }
        });
    }
}
