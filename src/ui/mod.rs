pub mod categorias;
pub mod components;
pub mod dashboard;
pub mod login;
pub mod movimentacoes;
pub mod produtos;
pub mod relatorios;
pub mod theme;
pub mod usuarios;

use egui::{Color32, CornerRadius, Frame, Layout, Margin};

use crate::domain::{PerfilUsuario, Usuario};

use self::categorias::CategoriasState;
use self::components::toast;
use self::dashboard::DashboardState;
use self::login::LoginState;
use self::movimentacoes::MovimentacoesState;
use self::produtos::ProdutosState;
use self::relatorios::RelatoriosState;
use self::theme::{apply_theme, Cores};
use self::usuarios::UsuariosState;

#[derive(PartialEq, Clone, Copy)]
pub enum Tela {
    Login,
    Dashboard,
    Produtos,
    Categorias,
    Movimentacoes,
    Relatorios,
    Usuarios,
}

pub struct App {
    pub conn: rusqlite::Connection,
    pub tela_atual: Tela,
    pub usuario_atual: Option<Usuario>,
    pub dark_mode: bool,

    pub login_state: LoginState,
    pub dashboard_state: DashboardState,
    pub produtos_state: ProdutosState,
    pub categorias_state: CategoriasState,
    pub movimentacoes_state: MovimentacoesState,
    pub relatorios_state: RelatoriosState,
    pub usuarios_state: UsuariosState,

    pub feedback: Option<(String, bool)>,
    pub feedback_timer: f32,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, conn: rusqlite::Connection) -> Self {
        Self {
            conn,
            tela_atual: Tela::Login,
            usuario_atual: None,
            dark_mode: false,
            login_state: LoginState::default(),
            dashboard_state: DashboardState::default(),
            produtos_state: ProdutosState::default(),
            categorias_state: CategoriasState::default(),
            movimentacoes_state: MovimentacoesState::default(),
            relatorios_state: RelatoriosState::default(),
            usuarios_state: UsuariosState::default(),
            feedback: None,
            feedback_timer: 0.0,
        }
    }

    pub fn on_navigate(&mut self, tela: Tela) {
        self.tela_atual = tela;
        match tela {
            Tela::Dashboard => self.dashboard_state.carregado = false,
            Tela::Produtos => self.produtos_state.carregado = false,
            Tela::Categorias => self.categorias_state.carregado = false,
            Tela::Movimentacoes => self.movimentacoes_state.carregado = false,
            Tela::Relatorios => {
                self.relatorios_state.carregado_estoque = false;
                self.relatorios_state.carregado_mensal = false;
            }
            Tela::Usuarios => self.usuarios_state.carregado = false,
            Tela::Login => {}
        }
    }

    pub fn set_feedback(&mut self, msg: &str, is_error: bool) {
        self.feedback = Some((msg.to_string(), is_error));
        self.feedback_timer = 3.0;
    }

    fn show_sidebar(&mut self, ctx: &egui::Context) {
        let eh_dona = self
            .usuario_atual
            .as_ref()
            .map(|u| u.perfil == PerfilUsuario::Dona)
            .unwrap_or(false);

        egui::SidePanel::left("nav_panel")
            .exact_width(185.0)
            .resizable(false)
            .frame(Frame::new().fill(Cores::SIDEBAR_BG))
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(Cores::SIDEBAR_TEXT);

                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("BOM PREÇO")
                            .size(18.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new("Gestão de Estoque")
                            .size(11.0)
                            .color(Color32::from_rgb(160, 185, 220)),
                    );
                });
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                let nav_items: &[(Tela, &str)] = &[
                    (Tela::Dashboard, "  Dashboard"),
                    (Tela::Produtos, "  Produtos"),
                    (Tela::Categorias, "  Categorias"),
                    (Tela::Movimentacoes, "  Movimentações"),
                ];
                for (tela, label) in nav_items {
                    let selected = self.tela_atual == *tela;
                    let resp = selectable_nav(ui, label, selected);
                    if resp.clicked() {
                        self.on_navigate(*tela);
                    }
                }
                if eh_dona {
                    let selected = self.tela_atual == Tela::Relatorios;
                    if selectable_nav(ui, "  Relatórios", selected).clicked() {
                        self.on_navigate(Tela::Relatorios);
                    }
                    let selected = self.tela_atual == Tela::Usuarios;
                    if selectable_nav(ui, "  Usuários", selected).clicked() {
                        self.on_navigate(Tela::Usuarios);
                    }
                }

                ui.with_layout(Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(12.0);

                    let icon = if self.dark_mode { "☀  Claro" } else { "🌙  Escuro" };
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(icon)
                                        .color(Color32::from_rgb(180, 200, 230))
                                        .size(13.0),
                                )
                                .fill(Color32::TRANSPARENT)
                                .min_size(egui::vec2(155.0, 26.0)),
                            )
                            .clicked()
                        {
                            self.dark_mode = !self.dark_mode;
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if let Some(u) = &self.usuario_atual.clone() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Sair").color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(80, 40, 40))
                                    .min_size(egui::vec2(155.0, 28.0)),
                                )
                                .clicked()
                            {
                                self.usuario_atual = None;
                                self.tela_atual = Tela::Login;
                            }
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(u.perfil.as_str())
                                    .size(11.0)
                                    .color(Color32::from_rgb(150, 175, 210)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(&u.nome).size(13.0).color(Color32::WHITE),
                            );
                        });
                    }
                });
            });
    }
}

fn selectable_nav(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let width = ui.available_width().max(160.0);
    let desired = egui::vec2(width, 34.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg = if selected {
            Cores::SIDEBAR_SEL
        } else if response.hovered() {
            Color32::from_rgb(55, 78, 110)
        } else {
            Cores::SIDEBAR_BG
        };
        let text_color = if selected { Color32::WHITE } else { Cores::SIDEBAR_TEXT };
        ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
        ui.painter().text(
            rect.left_center() + egui::vec2(12.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            text_color,
        );
    }

    response
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);

        if self.feedback.is_some() {
            self.feedback_timer -= ctx.input(|i| i.stable_dt);
            if self.feedback_timer <= 0.0 {
                self.feedback = None;
            }
        }

        if let Some((msg, is_error)) = &self.feedback.clone() {
            toast(ctx, msg, *is_error);
        }

        if self.tela_atual == Tela::Login {
            login::show(self, ctx);
            return;
        }

        self.show_sidebar(ctx);

        let tela = self.tela_atual;
        egui::CentralPanel::default()
            .frame(Frame::default().fill(Cores::FUNDO).inner_margin(Margin::same(20)))
            .show(ctx, |ui| match tela {
                Tela::Dashboard => dashboard::show(self, ui),
                Tela::Produtos => produtos::show(self, ui, ctx),
                Tela::Categorias => categorias::show(self, ui, ctx),
                Tela::Movimentacoes => movimentacoes::show(self, ui),
                Tela::Relatorios => relatorios::show(self, ui),
                Tela::Usuarios => usuarios::show(self, ui),
                Tela::Login => {}
            });
    }
}
