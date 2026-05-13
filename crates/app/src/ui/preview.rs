use egui::{Color32, RichText, Rounding, Stroke, Vec2};
use wallpaper_core::config::WallpaperEntry;
use crate::ui::theme;

pub enum PreviewAction {
    Apply,
    Pause,
    Remove,
}

pub struct PreviewPanel;

impl PreviewPanel {
    pub fn show(
        ui: &mut egui::Ui,
        active: Option<&WallpaperEntry>,
        fps: f32,
        is_running: bool,
        is_paused: bool,
    ) -> Option<PreviewAction> {
        let mut action = None;

        let avail = ui.available_size();
        let preview_h = (avail.y * 0.52).max(160.0).min(360.0);
        let preview_size = Vec2::new(avail.x, preview_h);

        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, Rounding::same(8.0), Color32::from_rgb(5, 0, 0));
            painter.rect_stroke(rect, Rounding::same(8.0), Stroke::new(1.5, theme::BORDER_BRIGHT));

            if let Some(entry) = active {
                let badge_text = format!("{} {}", entry.kind.icon(), entry.kind.label());
                let badge_rect = egui::Rect::from_min_size(
                    rect.min + Vec2::new(8.0, 8.0), Vec2::new(70.0, 20.0),
                );
                painter.rect_filled(badge_rect, Rounding::same(3.0), Color32::from_rgba_premultiplied(0,0,0,180));
                painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER,
                    badge_text, egui::FontId::proportional(10.5), theme::ACCENT);

                // Running indicator
                if is_running {
                    let status = if is_paused { "⏸ Pausado" } else { "● Ativo" };
                    let sc = if is_paused { theme::TEXT_DIM } else { theme::ACCENT };
                    painter.text(rect.max - Vec2::new(8.0, 24.0), egui::Align2::RIGHT_BOTTOM,
                        status, egui::FontId::monospace(9.0), sc);
                }

                painter.text(rect.max - Vec2::new(8.0, 10.0), egui::Align2::RIGHT_BOTTOM,
                    format!("{:.0} FPS", fps), egui::FontId::monospace(9.0), theme::ACCENT_DIM);

                painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                    &entry.name, egui::FontId::proportional(18.0), theme::TEXT);
                painter.text(rect.center() + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER,
                    &entry.author, egui::FontId::proportional(11.0), theme::TEXT_DIM);
            } else {
                painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                    "Nenhum wallpaper ativo", egui::FontId::proportional(14.0), theme::TEXT_FAINT);
                painter.text(rect.center() + Vec2::new(0.0, 20.0), egui::Align2::CENTER_CENTER,
                    "Selecione e clique em Aplicar", egui::FontId::proportional(11.0), theme::TEXT_FAINT);
            }
        }

        ui.add_space(8.0);

        // Control bar
        ui.horizontal(|ui| {
            if active.is_some() {
                // Apply / Re-apply button
                let apply_label = if is_running { "  Reaplicar  " } else { "  Aplicar  " };
                if theme::accent_button(ui, apply_label).clicked() {
                    action = Some(PreviewAction::Apply);
                }
                ui.add_space(4.0);

                if is_running {
                    let pause_label = if is_paused { "Retomar" } else { "Pausar" };
                    if theme::ghost_button(ui, pause_label).clicked() {
                        action = Some(PreviewAction::Pause);
                    }
                    ui.add_space(4.0);
                    if theme::ghost_button(ui, "Parar").clicked() {
                        action = Some(PreviewAction::Remove);
                    }
                }
            } else {
                ui.label(RichText::new("Selecione um wallpaper para aplicar")
                    .color(theme::TEXT_FAINT).size(12.0));
            }
        });

        action
    }
}
