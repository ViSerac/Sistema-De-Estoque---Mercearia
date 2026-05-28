use rusqlite::Connection;

use crate::domain::Usuario;
use crate::repository::usuario;

pub fn autenticar(
    conn: &Connection,
    login: &str,
    senha: &str,
) -> Result<Option<Usuario>, rusqlite::Error> {
    let hash = crate::util::hash_senha(senha);
    match usuario::buscar_por_login(conn, login)? {
        Some(u) if u.senha_hash == hash => Ok(Some(u)),
        _ => Ok(None),
    }
}
