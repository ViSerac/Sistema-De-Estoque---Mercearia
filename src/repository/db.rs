use rusqlite::Connection;

pub fn conectar() -> Result<Connection, rusqlite::Error> {
    // Resolve o db relativo ao executável para não depender do working directory
    let db_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("mercado.db")))
        .unwrap_or_else(|| std::path::PathBuf::from("mercado.db"));
    let conn = Connection::open(db_path)?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS categorias (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS produtos (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            nome             TEXT    NOT NULL,
            codigo_de_barras TEXT    NOT NULL,
            categoria        INTEGER REFERENCES categorias(id),
            preco_custo      REAL    NOT NULL,
            preco_de_venda   REAL    NOT NULL,
            estoque_minimo   INTEGER NOT NULL,
            quantidade_atual INTEGER NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_produtos_barcode
            ON produtos(codigo_de_barras);
        CREATE TABLE IF NOT EXISTS usuarios (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            nome   TEXT NOT NULL,
            login  TEXT NOT NULL UNIQUE,
            senha  TEXT NOT NULL,
            perfil TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS movimentacoes (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            tipo       TEXT    NOT NULL,
            quantidade INTEGER NOT NULL,
            data_hora  TEXT    NOT NULL,
            motivo     TEXT,
            produto_id INTEGER NOT NULL REFERENCES produtos(id),
            usuario_id INTEGER NOT NULL REFERENCES usuarios(id)
        );",
    )?;

    Ok(conn)
}

pub fn seed_demo(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM produtos", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // Categorias
    let cats: &[&str] = &[
        "Laticínios",
        "Bebidas",
        "Carnes e Aves",
        "Hortifruti",
        "Padaria e Confeitaria",
        "Mercearia",
        "Higiene Pessoal",
        "Limpeza",
        "Congelados",
        "Frios e Embutidos",
        "Bebidas Alcoólicas",
        "Snacks e Guloseimas",
    ];
    for nome in cats {
        conn.execute("INSERT INTO categorias (nome) VALUES (?1)", [nome])?;
    }

    // id de cada categoria na ordem de inserção
    let mut cat_ids: Vec<i64> = Vec::new();
    let mut stmt = conn.prepare("SELECT id FROM categorias ORDER BY id")?;
    let ids = stmt.query_map([], |r| r.get(0))?;
    for id in ids {
        cat_ids.push(id?);
    }
    let [lat, beb, car, hor, pad, mer, hig, lim, con, fri, alc, sna] = match cat_ids.as_slice() {
        [a, b, c, d, e, f, g, h, i, j, k, l, ..] => [*a, *b, *c, *d, *e, *f, *g, *h, *i, *j, *k, *l],
        _ => return Ok(()),
    };

    // (nome, categoria_id, preco_custo, preco_venda, estoque_min, qtd_atual)
    let produtos: &[(&str, i64, f64, f64, i64, i64)] = &[
        // Laticínios (17)
        ("Leite Integral 1L", lat, 3.50, 5.49, 30, 120),
        ("Leite Desnatado 1L", lat, 3.60, 5.69, 20, 85),
        ("Leite Semidesnatado 1L", lat, 3.55, 5.59, 15, 60),
        ("Iogurte Natural 170g", lat, 1.80, 2.99, 20, 75),
        ("Iogurte de Morango 170g", lat, 1.90, 3.19, 20, 68),
        ("Queijo Mussarela kg", lat, 22.00, 34.90, 5, 18),
        ("Queijo Prato kg", lat, 21.00, 33.50, 5, 14),
        ("Queijo Ricota 250g", lat, 5.50, 8.99, 8, 22),
        ("Requeijão Cremoso 250g", lat, 4.20, 6.99, 12, 40),
        ("Manteiga com Sal 200g", lat, 6.80, 10.90, 10, 35),
        ("Manteiga sem Sal 200g", lat, 6.80, 10.90, 8, 28),
        ("Creme de Leite 200g", lat, 2.90, 4.49, 15, 55),
        ("Leite Condensado 395g", lat, 4.50, 6.99, 15, 48),
        ("Nata 200g", lat, 3.80, 5.99, 10, 30),
        ("Queijo Coalho 400g", lat, 8.50, 13.90, 8, 25),
        ("Leite em Pó Integral 400g", lat, 11.00, 17.90, 10, 32),
        ("Bebida Láctea Morango 200ml", lat, 1.20, 1.99, 24, 72),
        // Bebidas (17)
        ("Água Mineral 500ml", beb, 0.90, 1.79, 48, 180),
        ("Água Mineral 1,5L", beb, 1.20, 2.19, 24, 96),
        ("Suco de Laranja 1L", beb, 4.50, 7.49, 12, 40),
        ("Suco de Uva 1L", beb, 5.20, 8.49, 12, 35),
        ("Refrigerante Cola 2L", beb, 4.80, 7.99, 24, 88),
        ("Refrigerante Limão 2L", beb, 4.60, 7.69, 12, 55),
        ("Refrigerante Guaraná 2L", beb, 4.50, 7.49, 24, 76),
        ("Chá Gelado Limão 1,5L", beb, 3.80, 6.29, 12, 40),
        ("Energético 250ml", beb, 4.20, 6.99, 12, 36),
        ("Isotônico Laranja 500ml", beb, 2.80, 4.79, 12, 30),
        ("Suco de Caixinha Maçã 200ml", beb, 1.10, 1.89, 24, 96),
        ("Café Solúvel 50g", beb, 5.50, 8.99, 10, 38),
        ("Café Torrado e Moído 250g", beb, 6.80, 10.99, 15, 52),
        ("Achocolatado 200ml", beb, 1.50, 2.49, 24, 84),
        ("Achocolatado 1L", beb, 5.50, 8.99, 12, 45),
        ("Erva-Mate 500g", beb, 5.20, 8.49, 8, 22),
        ("Chá de Camomila 10 sachês", beb, 2.80, 4.49, 12, 38),
        // Carnes e Aves (16)
        ("Frango Inteiro Congelado kg", car, 8.50, 13.90, 10, 28),
        ("Peito de Frango kg", car, 12.00, 19.90, 8, 22),
        ("Coxa e Sobrecoxa kg", car, 9.00, 14.90, 8, 30),
        ("Carne Moída kg", car, 18.00, 29.90, 5, 15),
        ("Acém kg", car, 16.00, 26.90, 5, 12),
        ("Costela Bovina kg", car, 22.00, 35.90, 5, 10),
        ("Picanha kg", car, 55.00, 89.90, 3, 8),
        ("Linguiça Calabresa kg", car, 14.00, 22.90, 5, 18),
        ("Filé de Tilápia kg", car, 18.00, 29.90, 5, 12),
        ("Camarão Médio kg", car, 35.00, 57.90, 3, 8),
        ("Carne de Porco kg", car, 14.00, 22.90, 5, 15),
        ("Pernil de Porco kg", car, 12.00, 19.90, 5, 14),
        ("Músculo Bovino kg", car, 17.00, 27.90, 5, 12),
        ("Coração de Frango kg", car, 8.00, 12.90, 5, 18),
        ("Ovo de Galinha Vermelho Dúzia", car, 5.50, 8.99, 12, 48),
        ("Ovo de Galinha Branco Dúzia", car, 5.20, 8.49, 12, 42),
        // Hortifruti (16)
        ("Banana Prata kg", hor, 2.50, 4.29, 5, 18),
        ("Maçã Fuji kg", hor, 5.80, 9.49, 5, 12),
        ("Laranja Pera kg", hor, 2.80, 4.69, 5, 20),
        ("Mamão Formosa kg", hor, 3.50, 5.79, 3, 8),
        ("Melancia kg", hor, 1.80, 2.99, 3, 6),
        ("Tomate Longa Vida kg", hor, 4.50, 7.49, 5, 15),
        ("Batata Inglesa kg", hor, 3.20, 5.29, 8, 25),
        ("Cebola kg", hor, 3.80, 6.29, 5, 18),
        ("Alho cabeça 100g", hor, 2.50, 4.19, 10, 35),
        ("Cenoura kg", hor, 3.50, 5.79, 5, 15),
        ("Alface Americana unidade", hor, 1.80, 2.99, 8, 22),
        ("Couve-Flor unidade", hor, 4.50, 7.49, 5, 10),
        ("Pepino kg", hor, 3.20, 5.29, 5, 12),
        ("Limão Tahiti kg", hor, 4.80, 7.99, 5, 14),
        ("Abacate kg", hor, 5.50, 8.99, 3, 8),
        ("Pimentão Vermelho kg", hor, 7.50, 12.49, 3, 10),
        // Padaria e Confeitaria (16)
        ("Pão Francês kg", pad, 7.00, 11.49, 3, 12),
        ("Pão de Forma 500g", pad, 4.80, 7.99, 8, 28),
        ("Pão de Queijo 400g", pad, 8.50, 13.99, 8, 22),
        ("Biscoito Água e Sal 200g", pad, 2.80, 4.59, 12, 42),
        ("Biscoito Recheado Chocolate 140g", pad, 2.50, 4.19, 12, 48),
        ("Biscoito Cream Cracker 200g", pad, 2.90, 4.79, 12, 38),
        ("Bolo de Chocolate 500g", pad, 9.00, 14.99, 5, 12),
        ("Wafer Baunilha 160g", pad, 2.20, 3.69, 12, 36),
        ("Farinha de Trigo 1kg", pad, 3.80, 6.29, 10, 35),
        ("Fermento Biológico 10g", pad, 1.20, 1.99, 12, 40),
        ("Açúcar Refinado 1kg", pad, 3.50, 5.79, 10, 38),
        ("Açúcar Mascavo 500g", pad, 4.20, 6.99, 8, 22),
        ("Margarina 500g", pad, 5.50, 8.99, 8, 28),
        ("Chocolate em Pó 200g", pad, 4.80, 7.99, 8, 25),
        ("Bolo de Fubá 400g", pad, 7.50, 12.49, 5, 10),
        ("Palha Italiana 300g", pad, 8.00, 12.99, 5, 12),
        // Mercearia (16)
        ("Arroz Branco 5kg", mer, 18.00, 29.90, 10, 38),
        ("Arroz Parboilizado 5kg", mer, 19.00, 31.90, 8, 28),
        ("Feijão Carioca 1kg", mer, 6.50, 10.99, 10, 35),
        ("Feijão Preto 1kg", mer, 6.80, 11.49, 8, 25),
        ("Macarrão Espaguete 500g", mer, 3.50, 5.79, 12, 45),
        ("Macarrão Penne 500g", mer, 3.80, 6.29, 10, 38),
        ("Óleo de Soja 900ml", mer, 5.50, 8.99, 12, 42),
        ("Azeite Extra Virgem 500ml", mer, 18.00, 29.90, 8, 22),
        ("Sal Refinado 1kg", mer, 1.80, 2.99, 15, 55),
        ("Molho de Tomate 340g", mer, 2.80, 4.59, 15, 52),
        ("Extrato de Tomate 340g", mer, 2.50, 4.19, 12, 40),
        ("Vinagre de Álcool 750ml", mer, 2.20, 3.69, 10, 32),
        ("Farinha de Mandioca 500g", mer, 3.20, 5.29, 10, 30),
        ("Flocos de Milho 500g", mer, 3.80, 6.29, 10, 28),
        ("Lentilha 500g", mer, 5.50, 8.99, 8, 20),
        ("Aveia em Flocos 250g", mer, 4.80, 7.99, 8, 25),
        // Higiene Pessoal (16)
        ("Sabonete Dove 90g", hig, 2.50, 4.19, 15, 55),
        ("Shampoo Seda 325ml", hig, 7.50, 12.49, 10, 32),
        ("Condicionador Seda 325ml", hig, 7.50, 12.49, 10, 28),
        ("Creme Dental Colgate 90g", hig, 3.80, 6.29, 12, 40),
        ("Escova de Dente Colgate", hig, 3.50, 5.79, 12, 38),
        ("Fio Dental 50m", hig, 2.80, 4.59, 10, 32),
        ("Desodorante Rexona 150ml", hig, 7.80, 12.99, 10, 30),
        ("Papel Higiênico 12 rolos", hig, 14.00, 22.90, 8, 25),
        ("Absorvente Sempre Livre 8un", hig, 4.50, 7.49, 10, 28),
        ("Loção Hidratante Nívea 200ml", hig, 9.50, 15.90, 8, 22),
        ("Água Micelar 200ml", hig, 12.00, 19.90, 5, 15),
        ("Talco Johnson 100g", hig, 5.80, 9.49, 8, 22),
        ("Algodão 50g", hig, 2.20, 3.69, 10, 32),
        ("Cotonete 75 hastes", hig, 2.50, 4.19, 10, 28),
        ("Barbeador Gillette 2un", hig, 5.50, 8.99, 8, 22),
        ("Creme de Barbear 65g", hig, 4.80, 7.99, 8, 18),
        // Limpeza (16)
        ("Detergente Ypê 500ml", lim, 1.80, 2.99, 15, 55),
        ("Esponja de Limpeza 3 un", lim, 2.20, 3.69, 12, 42),
        ("Água Sanitária 1L", lim, 2.50, 4.19, 12, 40),
        ("Desinfetante Pinho Sol 500ml", lim, 4.20, 6.99, 10, 32),
        ("Sabão em Pó Omo 1kg", lim, 10.00, 16.90, 8, 25),
        ("Amaciante Comfort 1L", lim, 7.50, 12.49, 8, 22),
        ("Limpador Multiuso Flash 500ml", lim, 4.80, 7.99, 10, 28),
        ("Limpador de Vidros 500ml", lim, 4.50, 7.49, 8, 20),
        ("Cloro em Pó 1kg", lim, 6.50, 10.99, 8, 18),
        ("Sacos de Lixo 30L 30 un", lim, 6.80, 11.49, 8, 22),
        ("Sacos de Lixo 100L 10 un", lim, 7.50, 12.49, 5, 15),
        ("Papel Toalha 2 rolos", lim, 5.50, 8.99, 8, 25),
        ("Luva de Borracha M", lim, 4.80, 7.99, 8, 20),
        ("Vassoura de Nylon", lim, 12.00, 19.90, 5, 10),
        ("Rodo 60cm", lim, 14.00, 22.90, 5, 8),
        ("Pá de Lixo", lim, 8.50, 13.90, 5, 10),
        // Congelados (8)
        ("Pizza Calabresa 460g", con, 12.00, 19.90, 5, 15),
        ("Lasanha à Bolonhesa 600g", con, 14.00, 22.90, 5, 12),
        ("Hambúrguer Bovino 4 un 480g", con, 11.00, 17.90, 8, 20),
        ("Nuggets de Frango 300g", con, 9.50, 15.90, 8, 22),
        ("Peixe Empanado 300g", con, 10.00, 16.90, 5, 14),
        ("Batata Palito Congelada 400g", con, 7.50, 12.49, 8, 25),
        ("Sorvete Creme 1,5L", con, 14.00, 22.90, 5, 12),
        ("Açaí Cremoso 500g", con, 12.00, 19.90, 5, 10),
        // Frios e Embutidos (8)
        ("Presunto Fatiado 200g", fri, 6.50, 10.90, 8, 25),
        ("Mortadela Fatiada 200g", fri, 5.50, 8.99, 8, 28),
        ("Salame Italiano 200g", fri, 9.50, 15.90, 5, 15),
        ("Peito de Peru Fatiado 200g", fri, 8.50, 13.90, 8, 20),
        ("Ovo de Codorna 24 un", fri, 4.80, 7.99, 8, 22),
        ("Mussarela Fatiada 200g", fri, 8.00, 12.99, 8, 18),
        ("Azeitona Verde 200g", fri, 5.50, 8.99, 8, 22),
        ("Creme Cheese 150g", fri, 7.50, 12.49, 8, 20),
        // Bebidas Alcoólicas (8)
        ("Cerveja Pilsen Lata 350ml", alc, 2.80, 4.59, 24, 96),
        ("Cerveja Long Neck 355ml", alc, 4.50, 7.49, 24, 72),
        ("Vinho Tinto Seco 750ml", alc, 22.00, 34.90, 5, 15),
        ("Vinho Branco Seco 750ml", alc, 22.00, 34.90, 5, 12),
        ("Cachaça 1L", alc, 12.00, 19.90, 5, 18),
        ("Vodka 1L", alc, 22.00, 34.90, 3, 8),
        ("Whisky 1L", alc, 65.00, 99.90, 3, 6),
        ("Sidra 660ml", alc, 8.50, 13.90, 5, 14),
        // Snacks e Guloseimas (8)
        ("Batata Chips Salgada 96g", sna, 3.50, 5.79, 12, 45),
        ("Amendoim Torrado Salgado 200g", sna, 3.80, 6.29, 12, 38),
        ("Pipoca Microondas Manteiga 100g", sna, 3.20, 5.29, 12, 40),
        ("Chocolate ao Leite 90g", sna, 4.20, 6.99, 12, 42),
        ("Barra de Cereal Morango 25g", sna, 1.50, 2.49, 15, 55),
        ("Chiclete Menta 15g", sna, 1.20, 1.99, 12, 48),
        ("Pirulito 15g", sna, 0.60, 0.99, 20, 80),
        ("Gelatina em Pó Morango 85g", sna, 2.20, 3.69, 10, 38),
    ];

    for (i, (nome, cat_id, custo, venda, est_min, qtd)) in produtos.iter().enumerate() {
        let barcode = format!("789{:010}", i + 1);
        conn.execute(
            "INSERT INTO produtos
             (nome, codigo_de_barras, categoria, preco_custo, preco_de_venda, estoque_minimo, quantidade_atual)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![nome, barcode, cat_id, custo, venda, est_min, qtd],
        )?;
    }

    Ok(())
}

pub fn seed_admin(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM usuarios WHERE perfil = 'Dona'",
        [],
        |row| row.get(0),
    )?;
    if count == 0 {
        let hash = crate::util::hash_senha("admin123");
        conn.execute(
            "INSERT INTO usuarios (nome, login, senha, perfil) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["Administradora", "admin", hash, "Dona"],
        )?;
    }
    Ok(())
}
