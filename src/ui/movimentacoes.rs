use egui_extras::{Column, TableBuilder};

use crate::domain::{Movimentacao, Produto, TipoMovimentacao};
use crate::repository::{movimentacao, produto};
use crate::service::estoque;

use super::components::{label_erro, secao_heading};
use super::theme::Cores;
use super::App;

#[derive(PartialEq, Clone, Copy)]
pub enum ModoMovimentacao {
    Historico,
    RegistrarEntrada,
    RegistrarSaida,
}

pub struct MovimentacoesState {
    pub movimentacoes: Vec<Movimentacao>,
    pub produtos: Vec<Produto>,
    pub modo: ModoMovimentacao,
    pub produto_id_form: i64,
    pub filtro_produto_form: String,
    pub quantidade_form: String,
    pub motivo_form: String,
    pub erro_form: Option<String>,
    pub carregado: bool,
}

impl Default for MovimentacoesState {
    fn default() -> Self {
        Self {
            movimentacoes: Vec::new(),
            produtos: Vec::new(),
            modo: ModoMovimentacao::Historico,
            produto_id_form: 0,
            filtro_produto_form: String::new(),
            quantidade_form: String::new(),
            motivo_form: String::new(),
            erro_form: None,
            carregado: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    if !app.movimentacoes_state.carregado {
        recarregar(app);
    }

    secao_heading(ui, "Movimentações de Estoque");

    ui.horizontal(|ui| {
        let modo = app.movimentacoes_state.modo;
        if ui
            .selectable_label(modo == ModoMovimentacao::Historico, "Histórico")
            .clicked()
        {
            app.movimentacoes_state.modo = ModoMovimentacao::Historico;
        }
        if ui
            .selectable_label(modo == ModoMovimentacao::RegistrarEntrada, "Registrar Entrada")
            .clicked()
        {
            app.movimentacoes_state.modo = ModoMovimentacao::RegistrarEntrada;
            app.movimentacoes_state.erro_form = None;
            app.movimentacoes_state.filtro_produto_form.clear();
        }
        if ui
            .selectable_label(modo == ModoMovimentacao::RegistrarSaida, "Registrar Saída")
            .clicked()
        {
            app.movimentacoes_state.modo = ModoMovimentacao::RegistrarSaida;
            app.movimentacoes_state.erro_form = None;
            app.movimentacoes_state.filtro_produto_form.clear();
        }
    });
    ui.separator();
    ui.add_space(8.0);

    match app.movimentacoes_state.modo {
        ModoMovimentacao::Historico => show_historico(app, ui),
        ModoMovimentacao::RegistrarEntrada => show_form(app, ui, TipoMovimentacao::Entrada),
        ModoMovimentacao::RegistrarSaida => show_form(app, ui, TipoMovimentacao::Saida),
    }
}

fn show_historico(app: &mut App, ui: &mut egui::Ui) {
    if app.movimentacoes_state.movimentacoes.is_empty() {
        ui.label("Nenhuma movimentação registrada.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("mov_hist_scroll")
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(110.0))
                .column(Column::initial(60.0))
                .column(Column::initial(180.0))
                .column(Column::initial(50.0))
                .column(Column::remainder().at_least(80.0))
                .column(Column::initial(100.0))
                .header(28.0, |mut h| {
                    h.col(|ui| { ui.strong("Data/Hora"); });
                    h.col(|ui| { ui.strong("Tipo"); });
                    h.col(|ui| { ui.strong("Produto"); });
                    h.col(|ui| { ui.strong("Qtd"); });
                    h.col(|ui| { ui.strong("Motivo"); });
                    h.col(|ui| { ui.strong("Usuário"); });
                })
                .body(|mut body| {
                    let movs = app.movimentacoes_state.movimentacoes.clone();
                    for m in &movs {
                        body.row(24.0, |mut row| {
                            row.col(|ui| {
                                ui.label(m.data_hora.format("%d/%m/%Y %H:%M").to_string());
                            });
                            row.col(|ui| {
                                let cor = match m.tipo {
                                    TipoMovimentacao::Entrada => Cores::VERDE,
                                    TipoMovimentacao::Saida => Cores::VERMELHO,
                                };
                                ui.colored_label(cor, m.tipo.as_str());
                            });
                            row.col(|ui| { ui.label(&m.produto_nome); });
                            row.col(|ui| { ui.label(m.quantidade.to_string()); });
                            row.col(|ui| { ui.label(&m.motivo); });
                            row.col(|ui| { ui.label(&m.usuario_nome); });
                        });
                    }
                });
        });
}

fn show_form(app: &mut App, ui: &mut egui::Ui, tipo: TipoMovimentacao) {
    let titulo = match tipo {
        TipoMovimentacao::Entrada => "Registrar Entrada de Estoque",
        TipoMovimentacao::Saida => "Registrar Saída de Estoque",
    };
    ui.label(egui::RichText::new(titulo).size(15.0).strong());
    ui.add_space(8.0);

    egui::Frame::default()
        .fill(ui.visuals().window_fill)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(20))
        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
        .show(ui, |ui| {
            egui::Grid::new("form_mov")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Produto *");
                    let prods = app.movimentacoes_state.produtos.clone();
                    let sel_nome = prods
                        .iter()
                        .find(|p| p.id == app.movimentacoes_state.produto_id_form)
                        .map(|p| format!("{} ({} un.)", p.nome, p.quantidade_atual))
                        .unwrap_or_else(|| "Selecione...".into());

                    let popup_id = ui.make_persistent_id("prod_sel_mov_popup");
                    let btn_resp = ui.add(
                        egui::Button::new(format!("{} ▾", sel_nome))
                            .min_size(egui::vec2(320.0, 24.0)),
                    );
                    if btn_resp.clicked() {
                        ui.memory_mut(|m| m.toggle_popup(popup_id));
                    }
                    egui::popup_below_widget(
                        ui,
                        popup_id,
                        &btn_resp,
                        egui::PopupCloseBehavior::CloseOnClickOutside,
                        |ui| {
                            ui.set_min_width(320.0);
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut app.movimentacoes_state.filtro_produto_form,
                                )
                                .hint_text("Filtrar produto...")
                                .desired_width(296.0),
                            );
                            ui.separator();
                            let filtro =
                                app.movimentacoes_state.filtro_produto_form.to_lowercase();
                            egui::ScrollArea::vertical()
                                .id_salt("prod_popup_scroll")
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for p in &prods {
                                        if filtro.is_empty()
                                            || p.nome.to_lowercase().contains(&filtro)
                                        {
                                            let label = format!(
                                                "{} ({} un.)",
                                                p.nome, p.quantidade_atual
                                            );
                                            let selected = p.id
                                                == app.movimentacoes_state.produto_id_form;
                                            if ui.selectable_label(selected, label).clicked() {
                                                app.movimentacoes_state.produto_id_form = p.id;
                                                app.movimentacoes_state
                                                    .filtro_produto_form
                                                    .clear();
                                                ui.memory_mut(|m| m.close_popup());
                                            }
                                        }
                                    }
                                });
                        },
                    );
                    ui.end_row();

                    ui.label("Quantidade *");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut app.movimentacoes_state.quantidade_form,
                        )
                        .desired_width(100.0)
                        .hint_text("0"),
                    );
                    ui.end_row();

                    ui.label("Motivo");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.movimentacoes_state.motivo_form)
                            .desired_width(320.0)
                            .hint_text("Opcional"),
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);
            label_erro(ui, &app.movimentacoes_state.erro_form.clone());

            let btn_cor = match tipo {
                TipoMovimentacao::Entrada => Cores::VERDE,
                TipoMovimentacao::Saida => Cores::VERMELHO,
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Registrar").color(egui::Color32::WHITE),
                    )
                    .fill(btn_cor),
                )
                .clicked()
            {
                registrar(app, tipo);
            }
        });
}

fn registrar(app: &mut App, tipo: TipoMovimentacao) {
    let produto_id = app.movimentacoes_state.produto_id_form;
    if produto_id == 0 {
        app.movimentacoes_state.erro_form = Some("Selecione um produto.".into());
        return;
    }
    let quantidade: i64 = match app.movimentacoes_state.quantidade_form.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            app.movimentacoes_state.erro_form = Some("Quantidade inválida.".into());
            return;
        }
    };
    let motivo = app.movimentacoes_state.motivo_form.trim().to_string();
    let usuario_id = app.usuario_atual.as_ref().map(|u| u.id).unwrap_or(0);
    let usuario_nome = app
        .usuario_atual
        .as_ref()
        .map(|u| u.nome.as_str())
        .unwrap_or("")
        .to_string();

    let resultado = match tipo {
        TipoMovimentacao::Entrada => estoque::registrar_entrada(
            &app.conn,
            produto_id,
            quantidade,
            &motivo,
            usuario_id,
            &usuario_nome,
        ),
        TipoMovimentacao::Saida => estoque::registrar_saida(
            &app.conn,
            produto_id,
            quantidade,
            &motivo,
            usuario_id,
            &usuario_nome,
        ),
    };

    match resultado {
        Ok(_) => {
            app.set_feedback("Movimentação registrada com sucesso.", false);
            app.movimentacoes_state.quantidade_form.clear();
            app.movimentacoes_state.motivo_form.clear();
            app.movimentacoes_state.produto_id_form = 0;
            app.movimentacoes_state.filtro_produto_form.clear();
            app.movimentacoes_state.erro_form = None;
            app.movimentacoes_state.modo = ModoMovimentacao::Historico;
            recarregar(app);
        }
        Err(e) => {
            app.movimentacoes_state.erro_form = Some(e.to_string());
        }
    }
}

fn recarregar(app: &mut App) {
    app.movimentacoes_state.movimentacoes =
        movimentacao::listar(&app.conn).unwrap_or_default();
    app.movimentacoes_state.produtos = produto::listar(&app.conn).unwrap_or_default();
    app.movimentacoes_state.carregado = true;
}
