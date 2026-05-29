mod domain;
mod repository;
mod service;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bom Preço — Gestão de Estoque")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mercearia",
        native_options,
        Box::new(|cc| {
            let conn = repository::db::conectar().expect("Falha ao conectar ao banco de dados");
            repository::db::seed_admin(&conn).expect("Falha ao criar usuário administrador");
            repository::db::seed_demo(&conn).expect("Falha ao popular dados de exemplo");
            repository::db::seed_estoque_baixo(&conn).expect("Falha ao popular estoque baixo");
            repository::db::seed_movimentacoes(&conn).expect("Falha ao popular movimentações de exemplo");
            Ok(Box::new(ui::App::new(cc, conn)))
        }),
    )
}
