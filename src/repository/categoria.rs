use rusqlite::Connection;

use crate::domain::Categoria;

pub fn listar(conn: &Connection) -> Result<Vec<Categoria>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, nome FROM categorias ORDER BY nome")?;
    let rows = stmt.query_map([], |row| {
        Ok(Categoria {
            id: row.get(0)?,
            nome: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn inserir(conn: &Connection, nome: &str) -> Result<i64, rusqlite::Error> {
    conn.execute("INSERT INTO categorias (nome) VALUES (?1)", [nome])?;
    Ok(conn.last_insert_rowid())
}

pub fn atualizar(conn: &Connection, id: i64, nome: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE categorias SET nome = ?1 WHERE id = ?2",
        rusqlite::params![nome, id],
    )?;
    Ok(())
}

pub fn deletar(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM categorias WHERE id = ?1", [id])?;
    Ok(())
}
