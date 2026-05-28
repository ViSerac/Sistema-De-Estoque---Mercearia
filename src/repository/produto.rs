use rusqlite::Connection;

use crate::domain::Produto;

const SELECT_PRODUTO: &str = "
    SELECT p.id, p.nome, p.codigo_de_barras, p.categoria,
           COALESCE(c.nome, ''), p.preco_custo, p.preco_de_venda,
           p.estoque_minimo, p.quantidade_atual
    FROM produtos p
    LEFT JOIN categorias c ON p.categoria = c.id";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Produto> {
    Ok(Produto {
        id: row.get(0)?,
        nome: row.get(1)?,
        codigo_de_barras: row.get(2)?,
        categoria_id: row.get(3)?,
        categoria_nome: row.get(4)?,
        preco_custo: row.get(5)?,
        preco_de_venda: row.get(6)?,
        estoque_minimo: row.get(7)?,
        quantidade_atual: row.get(8)?,
    })
}

pub fn listar(conn: &Connection) -> Result<Vec<Produto>, rusqlite::Error> {
    let sql = format!("{} ORDER BY p.nome", SELECT_PRODUTO);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn buscar_por_id(conn: &Connection, id: i64) -> Result<Option<Produto>, rusqlite::Error> {
    let sql = format!("{} WHERE p.id = ?1", SELECT_PRODUTO);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map([id], map_row)?;
    match rows.next() {
        Some(p) => Ok(Some(p?)),
        None => Ok(None),
    }
}

pub fn buscar_por_barcode(
    conn: &Connection,
    barcode: &str,
) -> Result<Option<Produto>, rusqlite::Error> {
    let sql = format!("{} WHERE p.codigo_de_barras = ?1", SELECT_PRODUTO);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map([barcode], map_row)?;
    match rows.next() {
        Some(p) => Ok(Some(p?)),
        None => Ok(None),
    }
}

pub fn inserir(conn: &Connection, p: &Produto) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO produtos
         (nome, codigo_de_barras, categoria, preco_custo, preco_de_venda, estoque_minimo, quantidade_atual)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            p.nome,
            p.codigo_de_barras,
            p.categoria_id,
            p.preco_custo,
            p.preco_de_venda,
            p.estoque_minimo,
            p.quantidade_atual
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn atualizar(conn: &Connection, p: &Produto) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE produtos SET nome=?1, codigo_de_barras=?2, categoria=?3,
         preco_custo=?4, preco_de_venda=?5, estoque_minimo=?6, quantidade_atual=?7
         WHERE id=?8",
        rusqlite::params![
            p.nome,
            p.codigo_de_barras,
            p.categoria_id,
            p.preco_custo,
            p.preco_de_venda,
            p.estoque_minimo,
            p.quantidade_atual,
            p.id
        ],
    )?;
    Ok(())
}

pub fn deletar(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM produtos WHERE id = ?1", [id])?;
    Ok(())
}

pub fn listar_estoque_baixo(conn: &Connection) -> Result<Vec<Produto>, rusqlite::Error> {
    let sql = format!(
        "{} WHERE p.quantidade_atual <= p.estoque_minimo ORDER BY p.nome",
        SELECT_PRODUTO
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn atualizar_quantidade(
    conn: &Connection,
    id: i64,
    nova_qtd: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE produtos SET quantidade_atual = ?1 WHERE id = ?2",
        rusqlite::params![nova_qtd, id],
    )?;
    Ok(())
}

pub fn contar(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM produtos", [], |row| row.get(0))
}

pub fn valor_total_estoque(conn: &Connection) -> Result<f64, rusqlite::Error> {
    conn.query_row(
        "SELECT COALESCE(SUM(quantidade_atual * preco_custo), 0.0) FROM produtos",
        [],
        |row| row.get(0),
    )
}
