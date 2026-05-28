use egui::{Align, Color32, CornerRadius, Frame, Key, Layout, Margin};

use super::theme::Cores;
use super::{App, Tela};
use crate::service::auth;

#[derive(Default)]
pub struct LoginState {
    pub login: String,
    pub senha: String,
    pub erro: Option<String>,
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(Frame::default().fill(Cores::FUNDO))
        .show(ctx, |ui| {
            let height = ui.available_height();
            ui.add_space(height * 0.15);
            ui.vertical_centered(|ui| {
                ui.set_max_width(360.0);

                Frame::default()
                    .fill(Cores::AZUL_PRIMARIO)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(20, 16))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("BOM PREÇO")
                                    .color(Color32::WHITE)
                                    .size(28.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("Sistema de Gestão de Estoque")
                                    .color(Color32::from_rgb(180, 210, 255))
                                    .size(13.0),
                            );
                        });
                    });

                ui.add_space(24.0);

                Frame::default()
                    .fill(Color32::WHITE)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(24))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(220, 225, 235)))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Entrar").size(18.0).strong());
                        ui.add_space(16.0);

                        egui::Grid::new("login_form")
                            .num_columns(2)
                            .spacing([12.0, 10.0])
                            .show(ui, |ui| {
                                ui.with_layout(
                                    Layout::right_to_left(Align::Center),
                                    |ui| ui.label("Login:"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.login_state.login)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("usuario"),
                                );
                                ui.end_row();

                                ui.with_layout(
                                    Layout::right_to_left(Align::Center),
                                    |ui| ui.label("Senha:"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.login_state.senha)
                                        .password(true)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("••••••••"),
                                );
                                ui.end_row();
                            });

                        ui.add_space(8.0);

                        if let Some(e) = &app.login_state.erro.clone() {
                            ui.colored_label(Cores::VERMELHO, e);
                            ui.add_space(4.0);
                        }

                        let enter = ctx.input(|i| i.key_pressed(Key::Enter));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Entrar").color(Color32::WHITE),
                                    )
                                    .fill(Cores::AZUL_PRIMARIO)
                                    .min_size(egui::vec2(90.0, 32.0)),
                                )
                                .clicked()
                                || enter
                            {
                                tentar_login(app);
                            }
                        });
                    });

                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Login padrão: admin / admin123")
                        .color(Color32::from_rgb(160, 170, 185))
                        .size(12.0),
                );
            });
        });
}

fn tentar_login(app: &mut App) {
    let login = app.login_state.login.trim().to_string();
    let senha = app.login_state.senha.clone();
    match auth::autenticar(&app.conn, &login, &senha) {
        Ok(Some(usuario)) => {
            app.usuario_atual = Some(usuario);
            app.login_state = LoginState::default();
            app.on_navigate(Tela::Dashboard);
        }
        Ok(None) => {
            app.login_state.erro = Some("Login ou senha incorretos.".into());
        }
        Err(e) => {
            app.login_state.erro = Some(format!("Erro: {}", e));
        }
    }
}
