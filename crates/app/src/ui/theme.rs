use egui::{Color32, FontId, FontFamily, Rounding, Stroke, Style, Visuals};

pub const BG:            Color32 = Color32::from_rgb(8,   8,   8  );
pub const BG_PANEL:      Color32 = Color32::from_rgb(14,  14,  14 );
pub const BG_SIDEBAR:    Color32 = Color32::from_rgb(10,  10,  10 );
pub const BG_ITEM:       Color32 = Color32::from_rgb(18,  18,  18 );
pub const BG_ITEM_HOVER: Color32 = Color32::from_rgb(28,  4,   4  );
pub const BG_ACTIVE:     Color32 = Color32::from_rgb(40,  4,   4  );
pub const ACCENT:        Color32 = Color32::from_rgb(200, 0,   0  );
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(255, 40,  40 );
pub const ACCENT_DIM:    Color32 = Color32::from_rgb(120, 0,   0  );
pub const TEXT:          Color32 = Color32::from_rgb(235, 235, 235);
pub const TEXT_DIM:      Color32 = Color32::from_rgb(130, 130, 130);
pub const TEXT_FAINT:    Color32 = Color32::from_rgb(60,  60,  60 );
pub const BORDER:        Color32 = Color32::from_rgb(35,  6,   6  );
pub const BORDER_BRIGHT: Color32 = Color32::from_rgb(80,  10,  10 );
pub const STATUS_BAR:    Color32 = Color32::from_rgb(6,   6,   6  );

pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();

    // Rounding
    style.visuals.window_rounding    = Rounding::same(6.0);
    style.visuals.menu_rounding      = Rounding::same(4.0);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    style.visuals.widgets.inactive.rounding       = Rounding::same(4.0);
    style.visuals.widgets.hovered.rounding        = Rounding::same(4.0);
    style.visuals.widgets.active.rounding         = Rounding::same(4.0);

    // Window/panel backgrounds
    style.visuals.window_fill   = BG_PANEL;
    style.visuals.panel_fill    = BG;
    style.visuals.extreme_bg_color = BG;
    style.visuals.code_bg_color = BG_ITEM;

    // Widgets — inactive
    style.visuals.widgets.noninteractive.bg_fill   = BG_ITEM;
    style.visuals.widgets.noninteractive.fg_stroke  = Stroke::new(1.0, TEXT_DIM);
    style.visuals.widgets.noninteractive.bg_stroke  = Stroke::new(1.0, BORDER);

    // Widgets — interactive
    style.visuals.widgets.inactive.bg_fill  = BG_ITEM;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);

    style.visuals.widgets.hovered.bg_fill   = BG_ITEM_HOVER;
    style.visuals.widgets.hovered.fg_stroke  = Stroke::new(1.0, TEXT);
    style.visuals.widgets.hovered.bg_stroke  = Stroke::new(1.0, ACCENT_DIM);

    style.visuals.widgets.active.bg_fill    = BG_ACTIVE;
    style.visuals.widgets.active.fg_stroke   = Stroke::new(1.0, ACCENT_BRIGHT);
    style.visuals.widgets.active.bg_stroke   = Stroke::new(1.0, ACCENT);

    style.visuals.widgets.open.bg_fill      = BG_ACTIVE;
    style.visuals.widgets.open.fg_stroke     = Stroke::new(1.0, ACCENT_BRIGHT);
    style.visuals.widgets.open.bg_stroke     = Stroke::new(1.0, ACCENT);

    // Selection
    style.visuals.selection.bg_fill   = BG_ACTIVE;
    style.visuals.selection.stroke    = Stroke::new(1.0, ACCENT);

    // Text
    style.visuals.override_text_color = Some(TEXT);

    // Hyperlinks
    style.visuals.hyperlink_color = ACCENT_BRIGHT;

    // Separator
    style.visuals.window_stroke = Stroke::new(1.0, BORDER);

    // Spacing
    style.spacing.item_spacing      = egui::vec2(8.0, 6.0);
    style.spacing.button_padding    = egui::vec2(12.0, 6.0);
    style.spacing.menu_margin       = egui::Margin::same(6.0);
    style.spacing.window_margin     = egui::Margin::same(12.0);

    ctx.set_style(style);
}

pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label).color(TEXT).strong(),
    )
    .fill(BG_ACTIVE)
    .stroke(Stroke::new(1.5, ACCENT));
    ui.add(btn)
}

pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label).color(TEXT_DIM),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(1.0, BORDER));
    ui.add(btn)
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(text).color(ACCENT_DIM).size(10.0).strong());
    ui.add_space(2.0);
}

pub fn separator(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(8.0));
}
