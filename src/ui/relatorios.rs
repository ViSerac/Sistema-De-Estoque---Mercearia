use chrono::Datelike;
use egui_extras::{Column, TableBuilder};

use crate::domain::{Movimentacao, Produto, TipoMovimentacao};
use crate::repository::{movimentacao, produto};

use super::components::{card_estatistica, secao_heading};
use super::theme::Cores;
use super::App;

#[derive(PartialEq)]
pub enum AbaRelatorio {
    EstoqueBaixo,
    Mensal,
}

pub struct RelatoriosState {
    pub estoque_baixo: Vec<Produto>,
    pub movimentacoes_mes: Vec<Movimentacao>,
    pub ano: i32,
    pub mes: u32,
    pub aba: AbaRelatorio,
    pub carregado_estoque: bool,
    pub carregado_mensal: bool,
}

impl Default for RelatoriosState {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            estoque_baixo: Vec::new(),
            movimentacoes_mes: Vec::new(),
            ano: now.year(),
            mes: now.month(),
            aba: AbaRelatorio::EstoqueBaixo,
            carregado_estoque: false,
            carregado_mensal: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    if !app.relatorios_state.carregado_estoque {
        app.relatorios_state.estoque_baixo =
            produto::listar_estoque_baixo(&app.conn).unwrap_or_default();
        app.relatorios_state.carregado_estoque = true;
    }

    secao_heading(ui, "Relatórios");

    let aba_atual = match app.relatorios_state.aba {
        AbaRelatorio::EstoqueBaixo => 0u8,
        AbaRelatorio::Mensal => 1u8,
    };
    ui.horizontal(|ui| {
        if ui.selectable_label(aba_atual == 0, "Estoque Baixo").clicked() {
            app.relatorios_state.aba = AbaRelatorio::EstoqueBaixo;
        }
        if ui.selectable_label(aba_atual == 1, "Relatório Mensal").clicked() {
            app.relatorios_state.aba = AbaRelatorio::Mensal;
        }
    });
    ui.separator();
    ui.add_space(8.0);

    match app.relatorios_state.aba {
        AbaRelatorio::EstoqueBaixo => show_estoque_baixo(app, ui),
        AbaRelatorio::Mensal => show_mensal(app, ui),
    }
}

fn show_estoque_baixo(app: &mut App, ui: &mut egui::Ui) {
    let count = app.relatorios_state.estoque_baixo.len();
    if count == 0 {
        ui.colored_label(Cores::VERDE, "Nenhum produto com estoque abaixo do mínimo.");
        return;
    }

    ui.colored_label(
        Cores::LARANJA,
        format!("{} produto(s) com estoque abaixo do mínimo", count),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .id_salt("rel_baixo_scroll")
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::remainder().at_least(160.0))
                .column(Column::initial(130.0))
                .column(Column::initial(90.0))
                .column(Column::initial(90.0))
                .column(Column::initial(90.0))
                .header(28.0, |mut h| {
                    h.col(|ui| { ui.strong("Produto"); });
                    h.col(|ui| { ui.strong("Categoria"); });
                    h.col(|ui| { ui.strong("Qtd Atual"); });
                    h.col(|ui| { ui.strong("Mínimo"); });
                    h.col(|ui| { ui.strong("Déficit"); });
                })
                .body(|mut body| {
                    let produtos = app.relatorios_state.estoque_baixo.clone();
                    for p in &produtos {
                        body.row(26.0, |mut row| {
                            row.col(|ui| {
                                ui.painter()
                                    .rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                ui.colored_label(Cores::LARANJA, &p.nome);
                            });
                            row.col(|ui| {
                                ui.painter()
                                    .rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                ui.label(&p.categoria_nome);
                            });
                            row.col(|ui| {
                                ui.painter()
                                    .rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                ui.colored_label(Cores::VERMELHO, p.quantidade_atual.to_string());
                            });
                            row.col(|ui| {
                                ui.painter()
                                    .rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                ui.label(p.estoque_minimo.to_string());
                            });
                            row.col(|ui| {
                                ui.painter()
                                    .rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                let deficit = p.estoque_minimo - p.quantidade_atual;
                                ui.colored_label(Cores::VERMELHO, deficit.to_string());
                            });
                        });
                    }
                });
        });
}

fn show_mensal(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Ano:");
        ui.add(egui::DragValue::new(&mut app.relatorios_state.ano).range(2020..=2035));
        ui.label("Mês:");
        ui.add(egui::DragValue::new(&mut app.relatorios_state.mes).range(1..=12));
        if ui.button("Carregar").clicked() {
            app.relatorios_state.movimentacoes_mes = movimentacao::listar_por_mes(
                &app.conn,
                app.relatorios_state.ano,
                app.relatorios_state.mes,
            )
            .unwrap_or_default();
            app.relatorios_state.carregado_mensal = true;
        }
    });
    ui.add_space(12.0);

    if !app.relatorios_state.carregado_mensal {
        ui.label("Selecione o período e clique em Carregar.");
        return;
    }

    let movs = &app.relatorios_state.movimentacoes_mes;
    let total_entradas: i64 = movs
        .iter()
        .filter(|m| m.tipo == TipoMovimentacao::Entrada)
        .map(|m| m.quantidade)
        .sum();
    let total_saidas: i64 = movs
        .iter()
        .filter(|m| m.tipo == TipoMovimentacao::Saida)
        .map(|m| m.quantidade)
        .sum();

    ui.horizontal(|ui| {
        card_estatistica(ui, "Total Entradas", &total_entradas.to_string(), Cores::VERDE);
        card_estatistica(ui, "Total Saídas", &total_saidas.to_string(), Cores::VERMELHO);
        card_estatistica(ui, "Registros", &movs.len().to_string(), Cores::AZUL_PRIMARIO);
    });
    ui.add_space(12.0);

    if movs.is_empty() {
        ui.label("Nenhuma movimentação neste período.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("rel_mensal_scroll")
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(130.0))
                .column(Column::initial(70.0))
                .column(Column::remainder().at_least(160.0))
                .column(Column::initial(60.0))
                .column(Column::initial(180.0))
                .header(28.0, |mut h| {
                    h.col(|ui| { ui.strong("Data/Hora"); });
                    h.col(|ui| { ui.strong("Tipo"); });
                    h.col(|ui| { ui.strong("Produto"); });
                    h.col(|ui| { ui.strong("Qtd"); });
                    h.col(|ui| { ui.strong("Motivo"); });
                })
                .body(|mut body| {
                    let movs_clone = app.relatorios_state.movimentacoes_mes.clone();
                    for m in &movs_clone {
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
                        });
                    }
                });
        });
}
