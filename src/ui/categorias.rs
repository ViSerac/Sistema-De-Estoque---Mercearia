use egui_extras::{Column, TableBuilder};

use crate::domain::{Categoria, PerfilUsuario};
use crate::repository::categoria;

use super::components::{label_erro, modal_confirmacao, secao_heading};
use super::theme::Cores;
use super::App;

pub struct CategoriasState {
    pub categorias: Vec<Categoria>,
    pub nome_form: String,
    pub editando_id: Option<i64>,
    pub confirmar_delete_id: Option<i64>,
    pub confirmando: bool,
    pub erro: Option<String>,
    pub carregado: bool,
}

impl Default for CategoriasState {
    fn default() -> Self {
        Self {
            categorias: Vec::new(),
            nome_form: String::new(),
            editando_id: None,
            confirmar_delete_id: None,
            confirmando: false,
            erro: None,
            carregado: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    if !app.categorias_state.carregado {
        recarregar(app);
    }

    secao_heading(ui, "Categorias");

    let editando = app.categorias_state.editando_id.is_some();

    egui::Frame::default()
        .fill(egui::Color32::WHITE)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(12))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 235)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(if editando { "Nome:" } else { "Nova categoria:" });
                ui.add(
                    egui::TextEdit::singleline(&mut app.categorias_state.nome_form)
                        .desired_width(220.0)
                        .hint_text("Nome da categoria"),
                );
                let btn_label = if editando { "Atualizar" } else { "Adicionar" };
                if ui.button(btn_label).clicked() {
                    salvar(app);
                }
                if editando && ui.button("Cancelar").clicked() {
                    app.categorias_state.editando_id = None;
                    app.categorias_state.nome_form.clear();
                    app.categorias_state.erro = None;
                }
            });
            label_erro(ui, &app.categorias_state.erro.clone());
        });

    ui.add_space(16.0);

    let eh_dona = app
        .usuario_atual
        .as_ref()
        .map(|u| u.perfil == PerfilUsuario::Dona)
        .unwrap_or(false);

    if app.categorias_state.categorias.is_empty() {
        ui.label("Nenhuma categoria cadastrada.");
    } else {
        let cats = app.categorias_state.categorias.clone();
        let mut editar: Option<Categoria> = None;
        let mut deletar: Option<i64> = None;

        egui::ScrollArea::vertical()
            .id_salt("cat_scroll")
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::remainder().at_least(120.0))
                    .column(Column::initial(60.0))
                    .column(Column::initial(65.0))
                    .body(|mut body| {
                        for cat in &cats {
                            body.row(28.0, |mut row| {
                                row.col(|ui| { ui.label(&cat.nome); });
                                row.col(|ui| {
                                    if ui.small_button("Editar").clicked() {
                                        editar = Some(cat.clone());
                                    }
                                });
                                row.col(|ui| {
                                    if eh_dona
                                        && ui
                                            .add(egui::Button::new("Excluir").fill(Cores::VERMELHO).small())
                                            .clicked()
                                    {
                                        deletar = Some(cat.id);
                                    }
                                });
                            });
                        }
                    });
            });

        if let Some(cat) = editar {
            app.categorias_state.editando_id = Some(cat.id);
            app.categorias_state.nome_form = cat.nome;
            app.categorias_state.erro = None;
        }
        if let Some(id) = deletar {
            app.categorias_state.confirmar_delete_id = Some(id);
            app.categorias_state.confirmando = true;
        }
    }

    let mut confirmando = app.categorias_state.confirmando;
    if modal_confirmacao(
        ctx,
        "Excluir Categoria",
        "Deseja realmente excluir esta categoria?",
        &mut confirmando,
    ) {
        if let Some(id) = app.categorias_state.confirmar_delete_id.take() {
            match categoria::deletar(&app.conn, id) {
                Ok(_) => {
                    app.set_feedback("Categoria excluída.", false);
                    recarregar(app);
                }
                Err(_) => {
                    app.set_feedback("Existem produtos nessa categoria.", true);
                }
            }
        }
    }
    app.categorias_state.confirmando = confirmando;
}

fn salvar(app: &mut App) {
    let nome = app.categorias_state.nome_form.trim().to_string();
    if nome.is_empty() {
        app.categorias_state.erro = Some("Nome não pode ser vazio.".into());
        return;
    }
    if let Some(id) = app.categorias_state.editando_id {
        match categoria::atualizar(&app.conn, id, &nome) {
            Ok(_) => {
                app.set_feedback("Categoria atualizada.", false);
                app.categorias_state.editando_id = None;
                app.categorias_state.nome_form.clear();
                app.categorias_state.erro = None;
                recarregar(app);
            }
            Err(e) => app.categorias_state.erro = Some(e.to_string()),
        }
    } else {
        match categoria::inserir(&app.conn, &nome) {
            Ok(_) => {
                app.set_feedback("Categoria adicionada.", false);
                app.categorias_state.nome_form.clear();
                app.categorias_state.erro = None;
                recarregar(app);
            }
            Err(e) => app.categorias_state.erro = Some(e.to_string()),
        }
    }
}

fn recarregar(app: &mut App) {
    app.categorias_state.categorias = categoria::listar(&app.conn).unwrap_or_default();
    app.categorias_state.carregado = true;
}
