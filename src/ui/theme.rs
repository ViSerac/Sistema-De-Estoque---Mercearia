use egui::{Color32, Vec2};

pub struct Cores;

impl Cores {
    pub const FUNDO: Color32 = Color32::from_rgb(245, 247, 250);
    pub const SIDEBAR_BG: Color32 = Color32::from_rgb(37, 55, 80);
    pub const SIDEBAR_TEXT: Color32 = Color32::from_rgb(210, 220, 235);
    pub const SIDEBAR_SEL: Color32 = Color32::from_rgb(52, 85, 130);
    pub const AZUL_PRIMARIO: Color32 = Color32::from_rgb(52, 100, 168);
    pub const VERMELHO: Color32 = Color32::from_rgb(200, 60, 50);
    pub const LARANJA: Color32 = Color32::from_rgb(220, 130, 30);
    pub const VERDE: Color32 = Color32::from_rgb(60, 160, 80);
    pub const LINHA_ALERTA: Color32 = Color32::from_rgb(255, 240, 225);
    pub const BRANCO: Color32 = Color32::WHITE;
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = Cores::FUNDO;
    visuals.window_fill = Color32::WHITE;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    style.spacing.interact_size.y = 28.0;
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    ctx.set_style(style);
}
