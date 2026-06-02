use egui::{Color32, Frame, Margin};

use crate::domain::{PerfilUsuario, Usuario};
use crate::repository::usuario;
use crate::util::hash_senha;

use super::components::{modal_confirmacao, secao_heading};
use super::theme::Cores;
use super::App;

#[derive(Default, PartialEq)]
pub enum ModoUsuarios {
    #[default]
    Lista,
    Novo,
    Editar,
}

#[derive(Default)]
pub struct UsuarioForm {
    pub nome: String,
    pub login: String,
    pub senha: String,
    pub senha_confirma: String,
}

pub struct UsuariosState {
    pub usuarios: Vec<Usuario>,
    pub modo: ModoUsuarios,
    pub form: UsuarioForm,
    pub editando_id: Option<i64>,
    pub confirmar_delete_id: Option<i64>,
    pub confirmando: bool,
    pub erro_form: Option<String>,
    pub carregado: bool,
}

impl Default for UsuariosState {
    fn default() -> Self {
        Self {
            usuarios: Vec::new(),
            modo: ModoUsuarios::Lista,
            form: UsuarioForm::default(),
            editando_id: None,
            confirmar_delete_id: None,
            confirmando: false,
            erro_form: None,
            carregado: false,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    if !app.usuarios_state.carregado {
        carregar(app);
    }

    match app.usuarios_state.modo {
        ModoUsuarios::Lista => show_lista(app, ui),
        ModoUsuarios::Novo | ModoUsuarios::Editar => show_form(app, ui),
    }
}

fn show_lista(app: &mut App, ui: &mut egui::Ui) {
    secao_heading(ui, "Usuários");

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("+ Novo Usuário").color(Color32::WHITE),
                    )
                    .fill(Cores::AZUL_PRIMARIO)
                    .min_size(egui::vec2(130.0, 28.0)),
                )
                .clicked()
            {
                app.usuarios_state.modo = ModoUsuarios::Novo;
                app.usuarios_state.editando_id = None;
                app.usuarios_state.form = UsuarioForm::default();
                app.usuarios_state.erro_form = None;
            }
        });
    });

    ui.add_space(12.0);

    let usuario_logado_id = app.usuario_atual.as_ref().map(|u| u.id).unwrap_or(0);
    let mut editar: Option<Usuario> = None;
    let mut deletar: Option<i64> = None;

    egui::ScrollArea::vertical()
        .id_salt("usuarios_scroll")
        .show(ui, |ui| {
            for u in app.usuarios_state.usuarios.clone() {
                Frame::default()
                    .fill(ui.visuals().window_fill)
                    .inner_margin(Margin::symmetric(12, 8))
                    .corner_radius(egui::CornerRadius::same(4))
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&u.nome).strong().size(14.0));
                                ui.label(
                                    egui::RichText::new(format!(
                                        "@{}  •  {}",
                                        u.login,
                                        u.perfil.as_str()
                                    ))
                                    .size(12.0)
                                    .color(Color32::from_rgb(120, 130, 145)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if u.perfil == PerfilUsuario::Funcionario
                                        && u.id != usuario_logado_id
                                    {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("Excluir")
                                                        .color(Color32::WHITE),
                                                )
                                                .fill(Cores::VERMELHO)
                                                .min_size(egui::vec2(70.0, 26.0)),
                                            )
                                            .clicked()
                                        {
                                            deletar = Some(u.id);
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new("Admin")
                                                .size(11.0)
                                                .color(Color32::from_rgb(150, 130, 60)),
                                        );
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new("Editar")
                                                .min_size(egui::vec2(60.0, 26.0)),
                                        )
                                        .clicked()
                                    {
                                        editar = Some(u.clone());
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });

    if let Some(u) = editar {
        app.usuarios_state.editando_id = Some(u.id);
        app.usuarios_state.form = UsuarioForm {
            nome: u.nome.clone(),
            login: u.login.clone(),
            senha: String::new(),
            senha_confirma: String::new(),
        };
        app.usuarios_state.erro_form = None;
        app.usuarios_state.modo = ModoUsuarios::Editar;
    }
    if let Some(id) = deletar {
        app.usuarios_state.confirmar_delete_id = Some(id);
        app.usuarios_state.confirmando = true;
    }

    if app.usuarios_state.confirmando {
        let confirmed = modal_confirmacao(
            ui.ctx(),
            "Excluir Usuário",
            "Tem certeza que deseja excluir este usuário?",
            &mut app.usuarios_state.confirmando,
        );
        if confirmed {
            if let Some(id) = app.usuarios_state.confirmar_delete_id {
                match usuario::deletar(&app.conn, id) {
                    Ok(_) => {
                        app.set_feedback("Usuário excluído.", false);
                        app.usuarios_state.carregado = false;
                    }
                    Err(e) => app.set_feedback(&format!("Erro: {}", e), true),
                }
            }
            app.usuarios_state.confirmar_delete_id = None;
        }
    }
}

fn show_form(app: &mut App, ui: &mut egui::Ui) {
    let editando = app.usuarios_state.modo == ModoUsuarios::Editar;
    let titulo = if editando { "Editar Usuário" } else { "Novo Usuário" };

    ui.horizontal(|ui| {
        if ui.button("← Voltar").clicked() {
            app.usuarios_state.modo = ModoUsuarios::Lista;
            app.usuarios_state.editando_id = None;
            app.usuarios_state.erro_form = None;
        }
    });
    ui.add_space(8.0);

    secao_heading(ui, titulo);

    Frame::default()
        .fill(ui.visuals().window_fill)
        .inner_margin(Margin::same(20))
        .corner_radius(egui::CornerRadius::same(6))
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, |ui| {
            egui::Grid::new("usuario_form")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("Nome:");
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut app.usuarios_state.form.nome)
                            .desired_width(260.0)
                            .hint_text("Nome completo"),
                    );
                    ui.end_row();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("Login:");
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut app.usuarios_state.form.login)
                            .desired_width(260.0)
                            .hint_text("nome.usuario"),
                    );
                    ui.end_row();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("Senha:");
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut app.usuarios_state.form.senha)
                            .password(true)
                            .desired_width(260.0)
                            .hint_text(if editando {
                                "Deixe vazio para manter"
                            } else {
                                "Mínimo 6 caracteres"
                            }),
                    );
                    ui.end_row();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("Confirmar senha:");
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut app.usuarios_state.form.senha_confirma)
                            .password(true)
                            .desired_width(260.0)
                            .hint_text(if editando {
                                "Deixe vazio para manter"
                            } else {
                                "Repita a senha"
                            }),
                    );
                    ui.end_row();

                    if !editando {
                        ui.label("");
                        ui.label(
                            egui::RichText::new("Perfil: Funcionário (fixo)")
                                .size(12.0)
                                .color(Color32::from_rgb(120, 130, 145)),
                        );
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);

            if let Some(e) = &app.usuarios_state.erro_form.clone() {
                ui.colored_label(Cores::VERMELHO, e);
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                let btn_label = if editando {
                    "Salvar Alterações"
                } else {
                    "Criar Usuário"
                };
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(btn_label).color(Color32::WHITE))
                            .fill(Cores::AZUL_PRIMARIO)
                            .min_size(egui::vec2(140.0, 30.0)),
                    )
                    .clicked()
                {
                    if editando {
                        atualizar_usuario(app);
                    } else {
                        salvar_usuario(app);
                    }
                }
                ui.add_space(8.0);
                if ui.button("Cancelar").clicked() {
                    app.usuarios_state.modo = ModoUsuarios::Lista;
                    app.usuarios_state.editando_id = None;
                    app.usuarios_state.erro_form = None;
                }
            });
        });
}

fn atualizar_usuario(app: &mut App) {
    let id = match app.usuarios_state.editando_id {
        Some(id) => id,
        None => return,
    };

    let nome = app.usuarios_state.form.nome.trim().to_string();
    let login = app.usuarios_state.form.login.trim().to_string();
    let senha = app.usuarios_state.form.senha.clone();
    let confirma = app.usuarios_state.form.senha_confirma.clone();

    if nome.is_empty() {
        app.usuarios_state.erro_form = Some("Nome é obrigatório.".into());
        return;
    }
    if login.is_empty() {
        app.usuarios_state.erro_form = Some("Login é obrigatório.".into());
        return;
    }
    if !senha.is_empty() {
        if senha.len() < 6 {
            app.usuarios_state.erro_form =
                Some("Senha deve ter pelo menos 6 caracteres.".into());
            return;
        }
        if senha != confirma {
            app.usuarios_state.erro_form = Some("As senhas não coincidem.".into());
            return;
        }
    }

    // Verifica login duplicado (ignora o próprio usuário)
    match usuario::buscar_por_login(&app.conn, &login) {
        Ok(Some(u)) if u.id != id => {
            app.usuarios_state.erro_form = Some("Este login já está em uso.".into());
            return;
        }
        Err(e) => {
            app.usuarios_state.erro_form = Some(format!("Erro: {}", e));
            return;
        }
        _ => {}
    }

    // Mantém dados existentes da lista em memória
    let usuario_atual = app.usuarios_state.usuarios.iter().find(|u| u.id == id).cloned();

    let senha_hash = if senha.is_empty() {
        usuario_atual
            .as_ref()
            .map(|u| u.senha_hash.clone())
            .unwrap_or_default()
    } else {
        hash_senha(&senha)
    };

    let perfil = usuario_atual
        .map(|u| u.perfil)
        .unwrap_or(PerfilUsuario::Funcionario);

    let atualizado = Usuario { id, nome, login, senha_hash, perfil };

    match usuario::atualizar(&app.conn, &atualizado) {
        Ok(_) => {
            app.set_feedback("Usuário atualizado com sucesso!", false);
            app.usuarios_state.modo = ModoUsuarios::Lista;
            app.usuarios_state.editando_id = None;
            app.usuarios_state.form = UsuarioForm::default();
            app.usuarios_state.erro_form = None;
            app.usuarios_state.carregado = false;
        }
        Err(e) => {
            app.usuarios_state.erro_form = Some(format!("Erro ao atualizar: {}", e));
        }
    }
}

fn salvar_usuario(app: &mut App) {
    let nome = app.usuarios_state.form.nome.trim().to_string();
    let login = app.usuarios_state.form.login.trim().to_string();
    let senha = app.usuarios_state.form.senha.clone();
    let confirma = app.usuarios_state.form.senha_confirma.clone();

    if nome.is_empty() {
        app.usuarios_state.erro_form = Some("Nome é obrigatório.".into());
        return;
    }
    if login.is_empty() {
        app.usuarios_state.erro_form = Some("Login é obrigatório.".into());
        return;
    }
    if senha.len() < 6 {
        app.usuarios_state.erro_form = Some("Senha deve ter pelo menos 6 caracteres.".into());
        return;
    }
    if senha != confirma {
        app.usuarios_state.erro_form = Some("As senhas não coincidem.".into());
        return;
    }

    match usuario::buscar_por_login(&app.conn, &login) {
        Ok(Some(_)) => {
            app.usuarios_state.erro_form = Some("Este login já está em uso.".into());
            return;
        }
        Err(e) => {
            app.usuarios_state.erro_form = Some(format!("Erro: {}", e));
            return;
        }
        Ok(None) => {}
    }

    let novo = Usuario {
        id: 0,
        nome,
        login,
        senha_hash: hash_senha(&senha),
        perfil: PerfilUsuario::Funcionario,
    };

    match usuario::inserir(&app.conn, &novo) {
        Ok(_) => {
            app.set_feedback("Usuário criado com sucesso!", false);
            app.usuarios_state.modo = ModoUsuarios::Lista;
            app.usuarios_state.form = UsuarioForm::default();
            app.usuarios_state.erro_form = None;
            app.usuarios_state.carregado = false;
        }
        Err(e) => {
            app.usuarios_state.erro_form = Some(format!("Erro ao criar usuário: {}", e));
        }
    }
}

fn carregar(app: &mut App) {
    app.usuarios_state.usuarios = usuario::listar(&app.conn).unwrap_or_default();
    app.usuarios_state.carregado = true;
}
