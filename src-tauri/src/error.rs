use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Единый тип ошибки бэкенда.
///
/// Сериализуется во фронтенд как `{ code, message }`, чтобы JS мог ветвиться
/// по машинному коду, а не парсить текст сообщения.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("a verified Microsoft account is required to use offline profiles")]
    LicenseRequired,

    #[error("authentication was cancelled")]
    AuthCancelled,

    #[error("the sign-in request expired, start it again")]
    AuthExpired,

    #[error("Microsoft client id is not configured; set ORBIT_MS_CLIENT_ID or config.json")]
    MissingClientId,

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("Microsoft authentication failed: {0}")]
    MicrosoftAuth(String),

    #[error("token storage is unavailable: {0}")]
    TokenStorage(String),

    #[error("Xbox Live authentication failed (code {code})")]
    XboxLive { code: u32 },

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("Modrinth API error: {0}")]
    Modrinth(String),

    #[error("file hash does not match the expected value")]
    HashMismatch,

    #[error("loader '{kind}' does not support Minecraft {mc}")]
    UnsupportedLoader { kind: String, mc: String },

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Машинный код для фронтенда. Стабильный контракт — не менять без причины.
    pub fn code(&self) -> &'static str {
        match self {
            Self::LicenseRequired => "license_required",
            Self::AuthCancelled => "auth_cancelled",
            Self::AuthExpired => "auth_expired",
            Self::MissingClientId => "missing_client_id",
            Self::Config(_) => "config_error",
            Self::MicrosoftAuth(_) => "microsoft_auth_error",
            Self::TokenStorage(_) => "token_storage_error",
            Self::XboxLive { .. } => "xbox_live_error",
            Self::Network(_) => "network_error",
            Self::Io(_) => "io_error",
            Self::InvalidData(_) => "invalid_data",
            Self::Modrinth(_) => "modrinth_error",
            Self::HashMismatch => "hash_mismatch",
            Self::UnsupportedLoader { .. } => "unsupported_loader",
            Self::Internal(_) => "internal_error",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
