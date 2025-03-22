use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status_code: u16,
    pub message: Option<String>,
    pub data: T,
}
