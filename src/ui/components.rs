use egui::{Align2, Color32, CornerRadius, Frame, Margin};

use super::theme::Cores;

pub fn card_estatistica(ui: &mut egui::Ui, titulo: &str, valor: &str, cor: Color32) {
    Frame::default()
        .fill(ui.visuals().window_fill)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(14))
        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
        .show(ui, |ui| {
            ui.set_min_width(130.0);
            ui.colored_label(cor, titulo);
            ui.label(egui::RichText::new(valor).size(26.0).strong());
        });
}

pub fn modal_confirmacao(
    ctx: &egui::Context,
    titulo: &str,
    mensagem: &str,
    aberto: &mut bool,
) -> bool {
    let mut confirmado = false;
    if !*aberto {
        return false;
    }
    egui::Window::new(titulo)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(mensagem);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Confirmar").fill(Cores::VERMELHO))
                    .clicked()
                {
                    confirmado = true;
                    *aberto = false;
                }
                if ui.button("Cancelar").clicked() {
                    *aberto = false;
                }
            });
        });
    confirmado
}

pub fn toast(ctx: &egui::Context, msg: &str, is_error: bool) {
    let bg = if is_error { Cores::VERMELHO } else { Cores::VERDE };
    egui::Area::new(egui::Id::new("toast_overlay"))
        .anchor(Align2::CENTER_BOTTOM, [0.0, -28.0])
        .show(ctx, |ui| {
            Frame::default()
                .fill(bg)
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(20, 10))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(msg).color(Color32::WHITE).size(14.0));
                });
        });
}

pub fn secao_heading(ui: &mut egui::Ui, texto: &str) {
    ui.add_space(2.0);
    ui.heading(texto);
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);
}

pub fn label_erro(ui: &mut egui::Ui, erro: &Option<String>) {
    if let Some(e) = erro {
        ui.colored_label(Cores::VERMELHO, e);
        ui.add_space(4.0);
    }
}
