use chrono::Datelike;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Plot};

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
    pub movimentos_por_dia: Vec<(String, i64, i64)>,
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
            movimentos_por_dia: Vec::new(),
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

    // Gráfico de déficit horizontal
    ui.label(egui::RichText::new("Déficit por Produto").size(13.0).strong());
    ui.add_space(4.0);

    let produtos_graf = app.relatorios_state.estoque_baixo.clone();
    let bars: Vec<Bar> = produtos_graf
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let deficit = (p.estoque_minimo - p.quantidade_atual).max(0) as f64;
            Bar::new(i as f64, deficit)
                .name(p.nome.clone())
                .fill(Cores::LARANJA)
                .width(0.7)
        })
        .collect();

    let nomes: Vec<(f64, String)> = produtos_graf
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let nome = if p.nome.len() > 16 {
                format!("{}…", &p.nome[..14])
            } else {
                p.nome.clone()
            };
            (i as f64, nome)
        })
        .collect();

    Plot::new("deficit_chart")
        .height(180.0)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .show_axes([true, true])
        .x_axis_formatter(move |mark, _range| {
            let x = mark.value.round() as usize;
            nomes.get(x).map(|(_, s)| s.clone()).unwrap_or_default()
        })
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(BarChart::new(bars));
        });

    ui.add_space(12.0);

    // Tabela detalhada
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
            app.relatorios_state.movimentos_por_dia = movimentacao::listar_por_dia_mes(
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

    // Gráfico de entradas vs saídas por dia
    let dados_dia = app.relatorios_state.movimentos_por_dia.clone();
    if !dados_dia.is_empty() {
        ui.label(egui::RichText::new("Entradas vs Saídas por Dia").size(13.0).strong());
        ui.add_space(4.0);

        let mut barras_e: Vec<Bar> = Vec::new();
        let mut barras_s: Vec<Bar> = Vec::new();
        let labels_dia: Vec<(f64, String)> = dados_dia
            .iter()
            .enumerate()
            .map(|(i, (dia, _, _))| (i as f64 * 2.0 + 0.45, dia.clone()))
            .collect();

        for (i, (_dia, ent, sai)) in dados_dia.iter().enumerate() {
            barras_e.push(
                Bar::new(i as f64 * 2.0, *ent as f64)
                    .width(0.85)
                    .fill(Cores::VERDE),
            );
            barras_s.push(
                Bar::new(i as f64 * 2.0 + 0.95, *sai as f64)
                    .width(0.85)
                    .fill(Cores::VERMELHO),
            );
        }

        Plot::new("mensal_chart")
            .height(180.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_axes([false, true])
            .x_axis_formatter(move |mark, _range| {
                let x = mark.value;
                labels_dia
                    .iter()
                    .min_by(|a, b| {
                        (a.0 - x).abs().partial_cmp(&(b.0 - x).abs()).unwrap()
                    })
                    .map(|(_, s)| format!("dia {}", s))
                    .unwrap_or_default()
            })
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(barras_e).name("Entradas"));
                plot_ui.bar_chart(BarChart::new(barras_s).name("Saídas"));
            });
        ui.add_space(8.0);
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
