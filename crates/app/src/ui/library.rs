use egui::{Color32, RichText, Rounding, Stroke, Vec2};
use wallpaper_core::config::{WallpaperEntry, WallpaperKind};
use crate::ui::theme;

pub struct LibraryPanel {
    pub selected: Option<String>,
    pub filter: String,
    pub kind_filter: Option<WallpaperKind>,
    pub wallpapers: Vec<WallpaperEntry>,
}

impl Default for LibraryPanel {
    fn default() -> Self {
        Self {
            selected: None,
            filter: String::new(),
            kind_filter: None,
            wallpapers: Vec::new(),
        }
    }
}

impl LibraryPanel {
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<WallpaperEntry> {
        let mut activated = None;

        // Search bar
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Buscar wallpapers...")
                    .desired_width(ui.available_width() - 110.0)
                    .frame(true),
            );
            if theme::ghost_button(ui, "+ Importar").clicked() {
                log::info!("Import wallpaper clicked");
            }
        });

        ui.add_space(6.0);

        // Kind filter chips
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filtrar:").color(theme::TEXT_FAINT).size(11.0));
            for kind in &[WallpaperKind::Shader, WallpaperKind::Video, WallpaperKind::Scene, WallpaperKind::Web] {
                let selected = self.kind_filter.as_ref() == Some(kind);
                let (fill, stroke) = if selected {
                    (theme::BG_ACTIVE, Stroke::new(1.0, theme::ACCENT))
                } else {
                    (theme::BG_ITEM, Stroke::new(1.0, theme::BORDER))
                };
                let label = format!("{} {}", kind.icon(), kind.label());
                let btn = ui.add(
                    egui::Button::new(RichText::new(&label).size(11.0))
                        .fill(fill)
                        .stroke(stroke)
                        .rounding(Rounding::same(3.0)),
                );
                if btn.clicked() {
                    if selected {
                        self.kind_filter = None;
                    } else {
                        self.kind_filter = Some(kind.clone());
                    }
                }
            }
        });

        ui.add_space(8.0);
        theme::separator(ui);
        ui.add_space(4.0);

        // Wallpaper grid
        let filtered: Vec<&WallpaperEntry> = self
            .wallpapers
            .iter()
            .filter(|w| {
                let name_match = self.filter.is_empty()
                    || w.name.to_lowercase().contains(&self.filter.to_lowercase());
                let kind_match = self
                    .kind_filter
                    .as_ref()
                    .map(|k| &w.kind == k)
                    .unwrap_or(true);
                name_match && kind_match
            })
            .collect();

        if filtered.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Nenhum wallpaper encontrado")
                        .color(theme::TEXT_FAINT)
                        .size(13.0),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Importe wallpapers para começar")
                        .color(theme::TEXT_FAINT)
                        .size(11.0),
                );
            });
            return None;
        }

        let card_size = Vec2::new(160.0, 120.0);
        let available = ui.available_width();
        let cols = ((available / (card_size.x + 8.0)) as usize).max(1);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let chunks: Vec<&[&WallpaperEntry]> = filtered.chunks(cols).collect();
            for row in chunks {
                ui.horizontal(|ui| {
                    for entry in row.iter() {
                        let is_selected = self.selected.as_deref() == Some(&entry.id);
                        let (border_col, bg) = if is_selected {
                            (theme::ACCENT, theme::BG_ACTIVE)
                        } else {
                            (theme::BORDER, theme::BG_ITEM)
                        };

                        let resp = ui.allocate_ui(card_size, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(
                                card_size,
                                egui::Sense::click(),
                            );
                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();
                                painter.rect_filled(rect, Rounding::same(6.0), bg);
                                painter.rect_stroke(
                                    rect,
                                    Rounding::same(6.0),
                                    Stroke::new(1.5, if resp.hovered() { theme::ACCENT_DIM } else { border_col }),
                                );

                                // Kind icon badge
                                let badge_rect = egui::Rect::from_min_size(
                                    rect.min + Vec2::new(6.0, 6.0),
                                    Vec2::new(48.0, 18.0),
                                );
                                painter.rect_filled(badge_rect, Rounding::same(3.0), theme::BG);
                                painter.text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("{} {}", entry.kind.icon(), entry.kind.label()),
                                    egui::FontId::proportional(10.0),
                                    theme::ACCENT_DIM,
                                );

                                // Thumbnail area (placeholder)
                                let thumb_rect = egui::Rect::from_min_max(
                                    rect.min + Vec2::new(0.0, 0.0),
                                    rect.max - Vec2::new(0.0, 32.0),
                                );
                                painter.rect_filled(thumb_rect, Rounding::same(6.0), Color32::from_rgb(12, 2, 2));

                                // Name
                                let name_y = rect.max.y - 24.0;
                                painter.text(
                                    egui::pos2(rect.min.x + 8.0, name_y),
                                    egui::Align2::LEFT_CENTER,
                                    &entry.name,
                                    egui::FontId::proportional(12.0),
                                    if is_selected { theme::ACCENT_BRIGHT } else { theme::TEXT },
                                );

                                // Author
                                painter.text(
                                    egui::pos2(rect.min.x + 8.0, rect.max.y - 10.0),
                                    egui::Align2::LEFT_CENTER,
                                    &entry.author,
                                    egui::FontId::proportional(10.0),
                                    theme::TEXT_DIM,
                                );
                            }
                            resp
                        });

                        if resp.inner.clicked() {
                            self.selected = Some(entry.id.clone());
        }
                        if resp.inner.double_clicked() {
                            activated = Some((*entry).clone());
                        }
                    }
                });
                ui.add_space(4.0);
            }
        });

        activated
    }
}
