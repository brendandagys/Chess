use std::fs;
use std::sync::OnceLock;

static OPENING_BOOK_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Extracts the embedded opening book to /tmp and returns the path.
/// Uses OnceLock to ensure this only happens once per Lambda container lifetime.
pub fn get_opening_book_path() -> Option<&'static str> {
    OPENING_BOOK_PATH
        .get_or_init(|| {
            const OPENING_BOOK_DATA: &[u8] = include_bytes!("../lpb-allbook.bin");

            let tmp_path = "/tmp/lpb-allbook.bin";

            match fs::write(tmp_path, OPENING_BOOK_DATA) {
                Ok(_) => {
                    tracing::info!("Successfully wrote opening book to {}", tmp_path);
                    Some(tmp_path.to_string())
                }
                Err(e) => {
                    tracing::error!("Failed to write opening book to /tmp: {}", e);
                    None
                }
            }
        })
        .as_deref()
}
