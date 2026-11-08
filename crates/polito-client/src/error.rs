#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentication required or expired token")]
    Unauthorized,
    #[error("resource not found")]
    NotFound,
    #[error("unexpected payload from API's response: {0}")]
    Malformed(&'static str),
    #[error("server error {status}: {message}")]
    Server { status: u16, message: String },
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
}