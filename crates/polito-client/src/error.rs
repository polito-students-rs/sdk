use polito_api::apis;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentication required or expired token")]
    Unauthorized,
    #[error("resource not found")]
    NotFound,
    #[error("request rejected {status}: {message}")]
    Rejected { status: u16, message: String },
    #[error("server error {status}: {message}")]
    Server { status: u16, message: String },
    #[error("unexpected payload from API's response: {0}")]
    Malformed(String),
    #[error("transport error")]
    Transport(#[from] reqwest::Error),
    #[error("middleware error: {0}")]
    Middleware(String),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("invalid base path {0:?}: expected an absolute http(s) URL")]
    InvalidBasePath(String),
    #[error("invalid MFA key")]
    InvalidMfaKey,
    #[error("failed to sign the MFA challenge")]
    FailedMfaSignature,
    #[error("MFA challenge belongs to another authenticator")]
    MfaDifferentSerial,
    #[error("MFA answer not accepted by the server")]
    MfaValidationFailed,
}

impl From<reqwest_middleware::Error> for ApiError {
    fn from(e: reqwest_middleware::Error) -> Self {
        match e {
            reqwest_middleware::Error::Reqwest(e) => Self::Transport(e),
            reqwest_middleware::Error::Middleware(e) => Self::Middleware(format!("{e:#}")),
        }
    }
}

/// Collapses the per-endpoint error of the generated code into [`ApiError`].
impl<T> From<apis::Error<T>> for ApiError {
    fn from(e: apis::Error<T>) -> Self {
        match e {
            apis::Error::Reqwest(e) => Self::Transport(e),
            apis::Error::ReqwestMiddleware(e) => e.into(),
            apis::Error::Serde(e) => Self::Malformed(e.to_string()),
            apis::Error::Io(e) => Self::Io(e),
            apis::Error::ResponseError(r) => {
                let status = r.status.as_u16();
                match status {
                    401 => Self::Unauthorized,
                    404 => Self::NotFound,
                    500..=599 => Self::Server {
                        status,
                        message: r.content,
                    },
                    _ => Self::Rejected {
                        status,
                        message: r.content,
                    },
                }
            }
        }
    }
}
