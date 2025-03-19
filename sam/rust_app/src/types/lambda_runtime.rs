use serde::Serialize;

#[derive(Serialize)]
pub struct CustomResponse {
    #[serde(rename = "status_code")]
    pub status_code: i32,
    pub body: String,
}
