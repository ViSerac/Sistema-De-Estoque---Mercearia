use sha2::{Digest, Sha256};

pub fn hash_senha(senha: &str) -> String {
    hex::encode(Sha256::digest(senha.as_bytes()))
}
