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
    });

    ui.add_space(20.0);

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
                .max_height(300.0)
                .show(col0, |ui| {
                    for p in &app.dashboard_state.estoque_baixo {
                        ui.horizontal(|ui| {
                            ui.colored_label(Cores::LARANJA, &p.nome);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!(
                                        "{}/{}",
                                        p.quantidade_atual, p.estoque_minimo
                                    ));
                                },
                            );
                        });
                        ui.separator();
                    }
                });
        }

        let col1 = &mut cols[1];
        col1.label(egui::RichText::new("Movimentações Recentes").size(15.0).strong());
        col1.separator();
        col1.add_space(4.0);

        if app.dashboard_state.movimentacoes_recentes.is_empty() {
            col1.label("Nenhuma movimentação registrada.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("dash_mov_scroll")
                .max_height(300.0)
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
                        ui.separator();
                    }
                });
        }
    });
}

fn carregar(app: &mut App) {
    app.dashboard_state.total_produtos =
        produto::contar(&app.conn).unwrap_or(0);
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
    app.dashboard_state.carregado = true;
}
