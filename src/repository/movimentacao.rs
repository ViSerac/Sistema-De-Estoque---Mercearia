use chrono::NaiveDateTime;
use rusqlite::Connection;

use crate::domain::{Movimentacao, TipoMovimentacao};

const FMT: &str = "%Y-%m-%d %H:%M:%S";

const SELECT_MOV: &str = "
    SELECT m.id, m.tipo, m.quantidade, m.data_hora, COALESCE(m.motivo, ''),
           m.produto_id, p.nome, m.usuario_id, u.nome
    FROM movimentacoes m
    JOIN produtos p ON m.produto_id = p.id
    JOIN usuarios u ON m.usuario_id = u.id";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Movimentacao> {
    let data_str: String = row.get(3)?;
    let data_hora = NaiveDateTime::parse_from_str(&data_str, FMT)
        .unwrap_or_default();
    Ok(Movimentacao {
        id: row.get(0)?,
        tipo: TipoMovimentacao::from_str(&row.get::<_, String>(1)?),
        quantidade: row.get(2)?,
        data_hora,
        motivo: row.get(4)?,
        produto_id: row.get(5)?,
        produto_nome: row.get(6)?,
        usuario_id: row.get(7)?,
        usuario_nome: row.get(8)?,
    })
}

pub fn inserir(conn: &Connection, m: &Movimentacao) -> Result<i64, rusqlite::Error> {
    let data_str = m.data_hora.format(FMT).to_string();
    conn.execute(
        "INSERT INTO movimentacoes (tipo, quantidade, data_hora, motivo, produto_id, usuario_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            m.tipo.as_str(),
            m.quantidade,
            data_str,
            m.motivo,
            m.produto_id,
            m.usuario_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn listar(conn: &Connection) -> Result<Vec<Movimentacao>, rusqlite::Error> {
    let sql = format!("{} ORDER BY m.data_hora DESC LIMIT 200", SELECT_MOV);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn listar_recentes(conn: &Connection, limit: i64) -> Result<Vec<Movimentacao>, rusqlite::Error> {
    let sql = format!("{} ORDER BY m.data_hora DESC LIMIT ?1", SELECT_MOV);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([limit], map_row)?;
    rows.collect()
}

pub fn listar_por_mes(
    conn: &Connection,
    ano: i32,
    mes: u32,
) -> Result<Vec<Movimentacao>, rusqlite::Error> {
    let sql = format!(
        "{} WHERE strftime('%Y', m.data_hora) = ?1
           AND strftime('%m', m.data_hora) = ?2
         ORDER BY m.data_hora DESC",
        SELECT_MOV
    );
    let ano_str = format!("{:04}", ano);
    let mes_str = format!("{:02}", mes);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![ano_str, mes_str], map_row)?;
    rows.collect()
}

/// Retorna (dia "dd/mm", entradas_un, saidas_un, lucro_R$) para os últimos `dias` dias.
pub fn listar_por_dia(
    conn: &Connection,
    dias: u32,
) -> Result<Vec<(String, i64, i64, f64)>, rusqlite::Error> {
    let sql = format!(
        "SELECT strftime('%d/%m', m.data_hora),
                SUM(CASE WHEN m.tipo='Entrada' THEN m.quantidade ELSE 0 END),
                SUM(CASE WHEN m.tipo='Saida'   THEN m.quantidade ELSE 0 END),
                SUM(CASE WHEN m.tipo='Saida'
                         THEN (p.preco_de_venda - p.preco_custo) * CAST(m.quantidade AS REAL)
                         ELSE 0.0 END)
         FROM movimentacoes m
         JOIN produtos p ON m.produto_id = p.id
         WHERE m.data_hora >= datetime('now', '-{} days')
         GROUP BY strftime('%Y-%m-%d', m.data_hora)
         ORDER BY strftime('%Y-%m-%d', m.data_hora)",
        dias
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;
    rows.collect()
}

/// Retorna (dia "dd", entradas_un, saidas_un, lucro_R$) para um mês/ano específico.
pub fn listar_por_dia_mes(
    conn: &Connection,
    ano: i32,
    mes: u32,
) -> Result<Vec<(String, i64, i64, f64)>, rusqlite::Error> {
    let ano_str = format!("{:04}", ano);
    let mes_str = format!("{:02}", mes);
    let mut stmt = conn.prepare(
        "SELECT strftime('%d', m.data_hora),
                SUM(CASE WHEN m.tipo='Entrada' THEN m.quantidade ELSE 0 END),
                SUM(CASE WHEN m.tipo='Saida'   THEN m.quantidade ELSE 0 END),
                SUM(CASE WHEN m.tipo='Saida'
                         THEN (p.preco_de_venda - p.preco_custo) * CAST(m.quantidade AS REAL)
                         ELSE 0.0 END)
         FROM movimentacoes m
         JOIN produtos p ON m.produto_id = p.id
         WHERE strftime('%Y', m.data_hora) = ?1 AND strftime('%m', m.data_hora) = ?2
         GROUP BY strftime('%d', m.data_hora)
         ORDER BY strftime('%d', m.data_hora)",
    )?;
    let rows = stmt.query_map(rusqlite::params![ano_str, mes_str], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;
    rows.collect()
}

pub fn contar_hoje(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM movimentacoes WHERE date(data_hora) = date('now')",
        [],
        |row| row.get(0),
    )
}
