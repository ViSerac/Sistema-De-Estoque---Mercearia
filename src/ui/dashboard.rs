use egui_plot::{Bar, BarChart, Plot};

use crate::domain::{Movimentacao, Produto, TipoMovimentacao};
use crate::repository::{movimentacao, produto};

use super::components::{card_estatistica, secao_heading};
use super::theme::Cores;
use super::App;

pub struct DashboardState {
    pub total_produtos: i64,
    pub total_categorias: i64,
    pub estoque_baixo: Vec<Produto>,
    pub movimentacoes_recentes: Vec<Movimentacao>,
    pub movimentacoes_hoje: i64,
    pub valor_total_estoque: f64,
    pub movimentos_7dias: Vec<(String, i64, i64)>,
    pub carregado: bool,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            total_produtos: 0,
            total_categorias: 0,
            estoque_baixo: Vec::new(),
            movimentacoes_recentes: Vec::new(),
            movimentacoes_hoje: 0,
            valor_total_estoque: 0.0,
            movimentos_7dias: Vec::new(),
            carregado: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    if !app.dashboard_state.carregado {
        carregar(app);
    }

    let s = &app.dashboard_state;

    secao_heading(ui, "Dashboard");

    // — Linha de cards —
    ui.horizontal(|ui| {
        card_estatistica(
            ui,
            "Total de Produtos",
            &s.total_produtos.to_string(),
            Cores::AZUL_PRIMARIO,
        );
        card_estatistica(
            ui,
            "Categorias",
            &s.total_categorias.to_string(),
            Cores::AZUL_PRIMARIO,
        );
        let cor_baixo = if s.estoque_baixo.is_empty() {
            Cores::VERDE
        } else {
            Cores::LARANJA
        };
        card_estatistica(
            ui,
            "Estoque Baixo",
            &s.estoque_baixo.len().to_string(),
            cor_baixo,
        );
        card_estatistica(
            ui,
            "Movim. Hoje",
            &s.movimentacoes_hoje.to_string(),
            Cores::VERDE,
        );
        card_estatistica(
            ui,
            "Valor em Estoque",
            &format!("R$ {:.0}", s.valor_total_estoque),
            egui::Color32::from_rgb(40, 80, 140),
        );
    });

    ui.add_space(20.0);

    // — Duas colunas: Estoque Baixo | Movimentações Recentes —
    ui.columns(2, |cols| {
        let col0 = &mut cols[0];
        col0.label(
            egui::RichText::new("Estoque Baixo")
                .size(15.0)
                .strong()
                .color(Cores::LARANJA),
        );
        col0.separator();
        col0.add_space(4.0);

        if app.dashboard_state.estoque_baixo.is_empty() {
            col0.colored_label(Cores::VERDE, "Nenhum produto com estoque baixo.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("dash_estoque_scroll")
                .max_height(220.0)
                .show(col0, |ui| {
                    for p in &app.dashboard_state.estoque_baixo {
                        ui.horizontal(|ui| {
                            ui.colored_label(Cores::LARANJA, &p.nome);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.colored_label(
                                        Cores::VERMELHO,
                                        format!("{}/{}", p.quantidade_atual, p.estoque_minimo),
                                    );
                                },
                            );
                        });
                        let ratio = if p.estoque_minimo > 0 {
                            (p.quantidade_atual as f32 / p.estoque_minimo as f32).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };
                        ui.add(egui::ProgressBar::new(ratio).fill(Cores::LARANJA));
                        ui.add_space(4.0);
                    }
                });
        }

        let col1 = &mut cols[1];
        col1.label(
            egui::RichText::new("Movimentações Recentes").size(15.0).strong(),
        );
        col1.separator();
        col1.add_space(4.0);

        if app.dashboard_state.movimentacoes_recentes.is_empty() {
            col1.label("Nenhuma movimentação registrada.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("dash_mov_scroll")
                .max_height(220.0)
                .show(col1, |ui| {
                    for m in &app.dashboard_state.movimentacoes_recentes {
                        ui.horizontal(|ui| {
                            let cor = match m.tipo {
                                TipoMovimentacao::Entrada => Cores::VERDE,
                                TipoMovimentacao::Saida => Cores::VERMELHO,
                            };
                            ui.colored_label(cor, m.tipo.as_str());
                            ui.label(&m.produto_nome);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!("x{}", m.quantidade));
                                },
                            );
                        });
                        ui.label(
                            egui::RichText::new(
                                m.data_hora.format("%d/%m %H:%M").to_string(),
                            )
                            .size(11.0)
                            .color(egui::Color32::from_rgb(140, 150, 165)),
                        );
                        ui.separator();
                    }
                });
        }
    });

    ui.add_space(16.0);

    // — Gráfico de atividade (fora do columns) —
    let dados = app.dashboard_state.movimentos_7dias.clone();
    if !dados.is_empty() {
        ui.label(
            egui::RichText::new("Atividade — Últimos 7 dias")
                .size(15.0)
                .strong(),
        );
        ui.separator();
        ui.add_space(4.0);

        let mut entradas: Vec<Bar> = Vec::new();
        let mut saidas: Vec<Bar> = Vec::new();
        for (i, (_dia, ent, sai)) in dados.iter().enumerate() {
            entradas.push(
                Bar::new(i as f64 * 2.0, *ent as f64)
                    .width(0.8)
                    .fill(Cores::VERDE),
            );
            saidas.push(
                Bar::new(i as f64 * 2.0 + 0.9, *sai as f64)
                    .width(0.8)
                    .fill(Cores::VERMELHO),
            );
        }

        let labels: Vec<(f64, String)> = dados
            .iter()
            .enumerate()
            .map(|(i, (dia, _, _))| (i as f64 * 2.0 + 0.45, dia.clone()))
            .collect();

        Plot::new("dash_7dias")
            .height(180.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .x_axis_formatter(move |mark, _range| {
                let x = mark.value;
                labels
                    .iter()
                    .min_by(|a, b| {
                        let da = (a.0 - x).abs();
                        let db = (b.0 - x).abs();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(_, s)| s.clone())
                    .unwrap_or_default()
            })
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(entradas).name("Entradas"));
                plot_ui.bar_chart(BarChart::new(saidas).name("Saídas"));
            });
    }
}

fn carregar(app: &mut App) {
    app.dashboard_state.total_produtos = produto::contar(&app.conn).unwrap_or(0);
    app.dashboard_state.total_categorias = app
        .conn
        .query_row("SELECT COUNT(*) FROM categorias", [], |r| r.get(0))
        .unwrap_or(0);
    app.dashboard_state.estoque_baixo =
        produto::listar_estoque_baixo(&app.conn).unwrap_or_default();
    app.dashboard_state.movimentacoes_recentes =
        movimentacao::listar_recentes(&app.conn, 10).unwrap_or_default();
    app.dashboard_state.movimentacoes_hoje =
        movimentacao::contar_hoje(&app.conn).unwrap_or(0);
    app.dashboard_state.valor_total_estoque =
        produto::valor_total_estoque(&app.conn).unwrap_or(0.0);
    app.dashboard_state.movimentos_7dias =
        movimentacao::listar_por_dia(&app.conn, 7).unwrap_or_default();
    app.dashboard_state.carregado = true;
}
