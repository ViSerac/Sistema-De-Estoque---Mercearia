use egui_extras::{Column, TableBuilder};

use crate::domain::{Categoria, PerfilUsuario, Produto};
use crate::repository::{categoria, produto};
use crate::service::estoque;

use super::components::{label_erro, modal_confirmacao, secao_heading};
use super::theme::Cores;
use super::{App, Tela};

#[derive(PartialEq, Clone, Copy)]
pub enum ModoProdutos {
    Lista,
    Cadastro,
    Edicao,
}

#[derive(Default, Clone)]
pub struct ProdutoForm {
    pub nome: String,
    pub codigo_de_barras: String,
    pub categoria_id: i64,
    pub preco_custo: String,
    pub preco_de_venda: String,
    pub estoque_minimo: String,
    pub quantidade_atual: String,
}

pub struct ProdutosState {
    pub produtos: Vec<Produto>,
    pub categorias: Vec<Categoria>,
    pub filtro: String,
    pub filtro_categoria_id: i64,
    pub modo: ModoProdutos,
    pub form: ProdutoForm,
    pub selecionado_id: Option<i64>,
    pub confirmar_delete: bool,
    pub erro_form: Option<String>,
    pub carregado: bool,
}

impl Default for ProdutosState {
    fn default() -> Self {
        Self {
            produtos: Vec::new(),
            categorias: Vec::new(),
            filtro: String::new(),
            filtro_categoria_id: 0,
            modo: ModoProdutos::Lista,
            form: ProdutoForm::default(),
            selecionado_id: None,
            confirmar_delete: false,
            erro_form: None,
            carregado: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    if !app.produtos_state.carregado {
        recarregar(app);
    }

    match app.produtos_state.modo {
        ModoProdutos::Lista => show_lista(app, ui, ctx),
        ModoProdutos::Cadastro | ModoProdutos::Edicao => show_form(app, ui),
    }
}

fn show_lista(app: &mut App, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.horizontal(|ui| {
        secao_heading(ui, "Produtos");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("+ Novo Produto").color(egui::Color32::WHITE),
                    )
                    .fill(Cores::AZUL_PRIMARIO),
                )
                .clicked()
            {
                app.produtos_state.modo = ModoProdutos::Cadastro;
                app.produtos_state.form = ProdutoForm::default();
                app.produtos_state.erro_form = None;
            }
        });
    });
    ui.add_space(2.0);

    if app.produtos_state.categorias.is_empty() {
        ui.horizontal(|ui| {
            ui.colored_label(
                Cores::LARANJA,
                "Nenhuma categoria cadastrada. Crie uma categoria primeiro.",
            );
            if ui.small_button("Ir para Categorias").clicked() {
                app.on_navigate(Tela::Categorias);
            }
        });
        ui.add_space(8.0);
    }

    ui.horizontal(|ui| {
        ui.label("Buscar:");
        ui.add(
            egui::TextEdit::singleline(&mut app.produtos_state.filtro)
                .desired_width(220.0)
                .hint_text("Nome ou código de barras"),
        );
        if ui.small_button("✕").clicked() {
            app.produtos_state.filtro.clear();
        }
        ui.add_space(12.0);
        ui.label("Categoria:");
        let cats = app.produtos_state.categorias.clone();
        let cat_label = if app.produtos_state.filtro_categoria_id == 0 {
            "Todas".to_string()
        } else {
            cats.iter()
                .find(|c| c.id == app.produtos_state.filtro_categoria_id)
                .map(|c| c.nome.clone())
                .unwrap_or_else(|| "Todas".to_string())
        };
        egui::ComboBox::from_id_salt("cat_filter")
            .selected_text(cat_label)
            .width(160.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.produtos_state.filtro_categoria_id, 0, "Todas");
                for c in &cats {
                    ui.selectable_value(
                        &mut app.produtos_state.filtro_categoria_id,
                        c.id,
                        &c.nome,
                    );
                }
            });
        if app.produtos_state.filtro_categoria_id != 0 && ui.small_button("✕").clicked() {
            app.produtos_state.filtro_categoria_id = 0;
        }
    });
    ui.add_space(8.0);

    let filtro = app.produtos_state.filtro.to_lowercase();
    let cat_id = app.produtos_state.filtro_categoria_id;
    let produtos: Vec<Produto> = app
        .produtos_state
        .produtos
        .iter()
        .filter(|p| {
            let texto_ok = filtro.is_empty()
                || p.nome.to_lowercase().contains(&filtro)
                || p.codigo_de_barras.contains(&filtro);
            let cat_ok = cat_id == 0 || p.categoria_id == cat_id;
            texto_ok && cat_ok
        })
        .cloned()
        .collect();

    if produtos.is_empty() {
        ui.centered_and_justified(|ui| ui.label("Nenhum produto encontrado."));
        return;
    }

    let eh_dona = app
        .usuario_atual
        .as_ref()
        .map(|u| u.perfil == PerfilUsuario::Dona)
        .unwrap_or(false);

    let mut editar_produto: Option<Produto> = None;
    let mut excluir_id: Option<i64> = None;

    egui::ScrollArea::vertical()
        .id_salt("prod_scroll")
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(200.0))
                .column(Column::initial(130.0))
                .column(Column::initial(120.0))
                .column(Column::initial(90.0))
                .column(Column::initial(90.0))
                .column(Column::initial(80.0))
                .column(Column::initial(110.0))
                .header(28.0, |mut h| {
                    h.col(|ui| { ui.strong("Nome"); });
                    h.col(|ui| { ui.strong("Cód. Barras"); });
                    h.col(|ui| { ui.strong("Categoria"); });
                    h.col(|ui| { ui.strong("Custo"); });
                    h.col(|ui| { ui.strong("Venda"); });
                    h.col(|ui| { ui.strong("Estoque"); });
                    h.col(|ui| { ui.strong("Ações"); });
                })
                .body(|mut body| {
                    for p in &produtos {
                        let baixo = p.estoque_baixo();
                        let p_clone = p.clone();
                        body.row(26.0, |mut row| {
                            row.col(|ui| {
                                if baixo {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA);
                                    ui.colored_label(Cores::LARANJA, &p_clone.nome);
                                } else {
                                    ui.label(&p_clone.nome);
                                }
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                ui.label(&p_clone.codigo_de_barras);
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                ui.label(&p_clone.categoria_nome);
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                ui.label(format!("R$ {:.2}", p_clone.preco_custo));
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                ui.label(format!("R$ {:.2}", p_clone.preco_de_venda));
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                let cor = if baixo { Cores::LARANJA } else { Cores::VERDE };
                                ui.colored_label(
                                    cor,
                                    format!("{}/{}", p_clone.quantidade_atual, p_clone.estoque_minimo),
                                );
                            });
                            row.col(|ui| {
                                if baixo { ui.painter().rect_filled(ui.max_rect(), 0.0, Cores::LINHA_ALERTA); }
                                ui.horizontal(|ui| {
                                    if ui.small_button("Editar").clicked() {
                                        editar_produto = Some(p_clone.clone());
                                    }
                                    if eh_dona
                                        && ui
                                            .add(
                                                egui::Button::new("Excluir")
                                                    .small()
                                                    .fill(egui::Color32::from_rgb(240, 220, 218)),
                                            )
                                            .clicked()
                                    {
                                        excluir_id = Some(p_clone.id);
                                    }
                                });
                            });
                        });
                    }
                });
        });

    if let Some(p) = editar_produto {
        abrir_edicao(app, &p);
    }
    if let Some(id) = excluir_id {
        app.produtos_state.selecionado_id = Some(id);
        app.produtos_state.confirmar_delete = true;
    }

    let mut del = app.produtos_state.confirmar_delete;
    if modal_confirmacao(
        ctx,
        "Excluir Produto",
        "Deseja realmente excluir este produto?\nEsta ação não pode ser desfeita.",
        &mut del,
    ) {
        if let Some(id) = app.produtos_state.selecionado_id.take() {
            match produto::deletar(&app.conn, id) {
                Ok(_) => {
                    app.set_feedback("Produto excluído.", false);
                    recarregar(app);
                }
                Err(_) => app.set_feedback("Não foi possível excluir o produto.", true),
            }
        }
    }
    app.produtos_state.confirmar_delete = del;
}

fn show_form(app: &mut App, ui: &mut egui::Ui) {
    let editando = app.produtos_state.modo == ModoProdutos::Edicao;

    ui.horizontal(|ui| {
        if ui.button("← Voltar").clicked() {
            app.produtos_state.modo = ModoProdutos::Lista;
            app.produtos_state.erro_form = None;
        }
        secao_heading(
            ui,
            if editando { "Editar Produto" } else { "Novo Produto" },
        );
    });
    ui.add_space(4.0);

    egui::Frame::default()
        .fill(ui.visuals().window_fill)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(20))
        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
        .show(ui, |ui| {
            egui::Grid::new("form_produto")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Nome *");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.produtos_state.form.nome)
                            .desired_width(280.0),
                    );
                    ui.end_row();

                    ui.label("Código de Barras *");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut app.produtos_state.form.codigo_de_barras,
                        )
                        .desired_width(280.0),
                    );
                    ui.end_row();

                    ui.label("Categoria *");
                    let cats = app.produtos_state.categorias.clone();
                    let sel_nome = cats
                        .iter()
                        .find(|c| c.id == app.produtos_state.form.categoria_id)
                        .map(|c| c.nome.as_str())
                        .unwrap_or("Selecione...");
                    egui::ComboBox::from_id_salt("cat_combo_prod")
                        .selected_text(sel_nome)
                        .width(280.0)
                        .show_ui(ui, |ui| {
                            for cat in &cats {
                                ui.selectable_value(
                                    &mut app.produtos_state.form.categoria_id,
                                    cat.id,
                                    &cat.nome,
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Preço de Custo *");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.produtos_state.form.preco_custo)
                            .desired_width(140.0)
                            .hint_text("Ex: 5,50"),
                    );
                    ui.end_row();

                    ui.label("Preço de Venda *");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.produtos_state.form.preco_de_venda)
                            .desired_width(140.0)
                            .hint_text("Ex: 8,90"),
                    );
                    ui.end_row();

                    ui.label("Estoque Mínimo");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.produtos_state.form.estoque_minimo)
                            .desired_width(80.0)
                            .hint_text("0"),
                    );
                    ui.end_row();

                    ui.label("Quantidade Atual");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut app.produtos_state.form.quantidade_atual,
                        )
                        .desired_width(80.0)
                        .hint_text("0"),
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);
            label_erro(ui, &app.produtos_state.erro_form.clone());

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Salvar").color(egui::Color32::WHITE),
                        )
                        .fill(Cores::AZUL_PRIMARIO),
                    )
                    .clicked()
                {
                    salvar_produto(app);
                }
                if ui.button("Cancelar").clicked() {
                    app.produtos_state.modo = ModoProdutos::Lista;
                    app.produtos_state.erro_form = None;
                }
            });
        });
}

fn abrir_edicao(app: &mut App, p: &Produto) {
    app.produtos_state.form = ProdutoForm {
        nome: p.nome.clone(),
        codigo_de_barras: p.codigo_de_barras.clone(),
        categoria_id: p.categoria_id,
        preco_custo: format!("{:.2}", p.preco_custo),
        preco_de_venda: format!("{:.2}", p.preco_de_venda),
        estoque_minimo: p.estoque_minimo.to_string(),
        quantidade_atual: p.quantidade_atual.to_string(),
    };
    app.produtos_state.selecionado_id = Some(p.id);
    app.produtos_state.modo = ModoProdutos::Edicao;
    app.produtos_state.erro_form = None;
}

fn salvar_produto(app: &mut App) {
    let form = &app.produtos_state.form;
    let nome = form.nome.trim().to_string();
    let barcode = form.codigo_de_barras.trim().to_string();

    if nome.is_empty() || barcode.is_empty() || form.categoria_id == 0 {
        app.produtos_state.erro_form = Some("Preencha todos os campos obrigatórios.".into());
        return;
    }

    let parse_f64 = |s: &str| s.replace(',', ".").parse::<f64>();
    let parse_i64 = |s: &str| s.trim().parse::<i64>();

    let preco_custo = match parse_f64(form.preco_custo.trim()) {
        Ok(v) => v,
        Err(_) => {
            app.produtos_state.erro_form = Some("Preço de custo inválido.".into());
            return;
        }
    };
    let preco_venda = match parse_f64(form.preco_de_venda.trim()) {
        Ok(v) => v,
        Err(_) => {
            app.produtos_state.erro_form = Some("Preço de venda inválido.".into());
            return;
        }
    };
    let estoque_min = parse_i64(&form.estoque_minimo).unwrap_or(0);
    let qtd_atual = parse_i64(&form.quantidade_atual).unwrap_or(0);

    let id = if app.produtos_state.modo == ModoProdutos::Edicao {
        app.produtos_state.selecionado_id.unwrap_or(0)
    } else {
        0
    };

    let p = Produto {
        id,
        nome,
        codigo_de_barras: barcode,
        categoria_id: form.categoria_id,
        categoria_nome: String::new(),
        preco_custo,
        preco_de_venda: preco_venda,
        estoque_minimo: estoque_min,
        quantidade_atual: qtd_atual,
    };

    if let Err(e) = estoque::validar_produto(&app.conn, &p) {
        app.produtos_state.erro_form = Some(e.to_string());
        return;
    }

    let resultado = if id == 0 {
        produto::inserir(&app.conn, &p).map(|_| ())
    } else {
        produto::atualizar(&app.conn, &p)
    };

    match resultado {
        Ok(_) => {
            let msg = if id == 0 {
                "Produto cadastrado com sucesso."
            } else {
                "Produto atualizado com sucesso."
            };
            app.set_feedback(msg, false);
            app.produtos_state.modo = ModoProdutos::Lista;
            app.produtos_state.erro_form = None;
            recarregar(app);
        }
        Err(e) => {
            app.produtos_state.erro_form = Some(e.to_string());
        }
    }
}

fn recarregar(app: &mut App) {
    app.produtos_state.produtos = produto::listar(&app.conn).unwrap_or_default();
    app.produtos_state.categorias = categoria::listar(&app.conn).unwrap_or_default();
    app.produtos_state.carregado = true;
}
