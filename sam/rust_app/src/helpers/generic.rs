use base64::{engine::general_purpose, Engine as _};
use rand::{distr::Alphanumeric, Rng};

pub fn generate_id() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

pub fn bytes_to_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

pub fn base64_to_bytes(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(s)
}
