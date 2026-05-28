use rusqlite::Connection;

use crate::domain::{PerfilUsuario, Usuario};

pub fn listar(conn: &Connection) -> Result<Vec<Usuario>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, nome, login, senha, perfil FROM usuarios ORDER BY nome")?;
    let rows = stmt.query_map([], |row| {
        Ok(Usuario {
            id: row.get(0)?,
            nome: row.get(1)?,
            login: row.get(2)?,
            senha_hash: row.get(3)?,
            perfil: PerfilUsuario::from_str(&row.get::<_, String>(4)?),
        })
    })?;
    rows.collect()
}

pub fn buscar_por_login(
    conn: &Connection,
    login: &str,
) -> Result<Option<Usuario>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, nome, login, senha, perfil FROM usuarios WHERE login = ?1",
    )?;
    let mut rows = stmt.query_map([login], |row| {
        Ok(Usuario {
            id: row.get(0)?,
            nome: row.get(1)?,
            login: row.get(2)?,
            senha_hash: row.get(3)?,
            perfil: PerfilUsuario::from_str(&row.get::<_, String>(4)?),
        })
    })?;
    match rows.next() {
        Some(u) => Ok(Some(u?)),
        None => Ok(None),
    }
}

pub fn inserir(conn: &Connection, usuario: &Usuario) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO usuarios (nome, login, senha, perfil) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            usuario.nome,
            usuario.login,
            usuario.senha_hash,
            usuario.perfil.as_str()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn atualizar(conn: &Connection, usuario: &Usuario) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE usuarios SET nome = ?1, login = ?2, senha = ?3, perfil = ?4 WHERE id = ?5",
        rusqlite::params![
            usuario.nome,
            usuario.login,
            usuario.senha_hash,
            usuario.perfil.as_str(),
            usuario.id
        ],
    )?;
    Ok(())
}

pub fn deletar(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM usuarios WHERE id = ?1", [id])?;
    Ok(())
}
