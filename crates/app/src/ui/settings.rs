use egui::RichText;
use wallpaper_core::config::{AppConfig, GpuQuality, StretchMode};
use crate::ui::theme;

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn show(ui: &mut egui::Ui, config: &mut AppConfig) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── Performance ──────────────────────────────────────────────
            theme::section_label(ui, "DESEMPENHO");

            ui.add_space(4.0);

            // FPS slider
            ui.horizontal(|ui| {
                ui.label("FPS alvo");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} fps", config.performance.target_fps))
                            .color(theme::ACCENT)
                            .strong(),
                    );
                });
            });
            ui.add(
                egui::Slider::new(&mut config.performance.target_fps, 1..=240)
                    .show_value(false)
                    .clamp_to_range(true),
            );

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("FPS em idle");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} fps", config.performance.idle_fps))
                            .color(theme::ACCENT_DIM),
                    );
                });
            });
            ui.add(
                egui::Slider::new(&mut config.performance.idle_fps, 1..=30)
                    .show_value(false)
                    .clamp_to_range(true),
            );

            ui.add_space(8.0);

            // GPU quality
            ui.label("Qualidade GPU");
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                for q in &[GpuQuality::Low, GpuQuality::Medium, GpuQuality::High, GpuQuality::Ultra] {
                    let selected = &config.performance.gpu_quality == q;
                    let resp = ui.selectable_label(
                        selected,
                        RichText::new(q.label())
                            .color(if selected { theme::ACCENT_BRIGHT } else { theme::TEXT_DIM })
                            .size(12.0),
                    );
                    if resp.clicked() {
                        config.performance.gpu_quality = q.clone();
                    }
                }
            });

            ui.add_space(10.0);

            // Toggles
            ui.checkbox(
                &mut config.performance.pause_on_fullscreen,
                RichText::new("Pausar com app em tela cheia").color(theme::TEXT),
            );
            ui.checkbox(
                &mut config.performance.reduce_on_battery,
                RichText::new("Reduzir FPS com bateria").color(theme::TEXT),
            );

            ui.add_space(16.0);
            theme::separator(ui);

            // ── Display ───────────────────────────────────────────────────
            theme::section_label(ui, "EXIBIÇÃO");
            ui.add_space(4.0);

            ui.label("Monitor");
            egui::ComboBox::from_id_salt("monitor_select")
                .selected_text(format!("Monitor {}", config.display.monitor_index + 1))
                .show_ui(ui, |ui| {
                    for i in 0..4usize {
                        ui.selectable_value(
                            &mut config.display.monitor_index,
                            i,
                            format!("Monitor {}", i + 1),
                        );
                    }
                });

            ui.add_space(6.0);

            ui.label("Modo de ajuste");
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                for mode in &[
                    StretchMode::Fill,
                    StretchMode::Fit,
                    StretchMode::Stretch,
                    StretchMode::Tile,
                    StretchMode::Center,
                ] {
                    let selected = &config.display.stretch_mode == mode;
                    let resp = ui.selectable_label(
                        selected,
                        RichText::new(mode.label())
                            .color(if selected { theme::ACCENT_BRIGHT } else { theme::TEXT_DIM })
                            .size(12.0),
                    );
                    if resp.clicked() {
                        config.display.stretch_mode = mode.clone();
                    }
                }
            });

            ui.add_space(16.0);
            theme::separator(ui);

            // ── Library path ──────────────────────────────────────────────
            theme::section_label(ui, "BIBLIOTECA");
            ui.add_space(4.0);

            let path_str = config.library_path.to_string_lossy().to_string();
            let mut path_edit = path_str.clone();
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut path_edit)
                        .desired_width(ui.available_width() - 80.0)
                        .hint_text("Caminho da biblioteca"),
                );
                if theme::ghost_button(ui, "Procurar").clicked() {
                    log::info!("Browse library path");
                }
            });

            ui.add_space(20.0);

            // Save button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if theme::accent_button(ui, "Salvar configurações").clicked() {
                    log::info!("Saving config");
                }
            });
        });
    }
}
