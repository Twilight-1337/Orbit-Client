use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::paths::AppPaths;

/// Переменная окружения имеет приоритет над локальным `config.json`.
pub const CLIENT_ID_ENV: &str = "ORBIT_MS_CLIENT_ID";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ConfigFile {
    microsoft_client_id: Option<String>,
}

/// Конфигурация приложения.
///
/// `client_id` — публичный идентификатор Azure-приложения с включённым
/// Device Code Flow. Секрета у публичного клиента нет, но в репозиторий его
/// всё равно не кладём: значение приходит из env или `config.json`.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub microsoft_client_id: String,
}

impl Config {
    pub fn load(paths: &AppPaths) -> AppResult<Self> {
        let from_file = read_config_file(paths)?;

        let client_id = std::env::var(CLIENT_ID_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or(from_file)
            .unwrap_or_default();

        if !client_id.is_empty() {
            uuid::Uuid::parse_str(&client_id).map_err(|_| {
                AppError::Config(format!(
                    "microsoftClientId must be a UUID (got '{client_id}')"
                ))
            })?;
        }

        Ok(Self {
            microsoft_client_id: client_id,
        })
    }

    pub fn require_client_id(&self) -> AppResult<&str> {
        if self.microsoft_client_id.is_empty() {
            return Err(AppError::MissingClientId);
        }

        Ok(&self.microsoft_client_id)
    }
}

fn read_config_file(paths: &AppPaths) -> AppResult<Option<String>> {
    let path = paths.config_file();
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read(&path)?;
    let parsed: ConfigFile = serde_json::from_slice(&raw)
        .map_err(|err| AppError::Config(format!("config.json is invalid: {err}")))?;

    Ok(parsed
        .microsoft_client_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}
