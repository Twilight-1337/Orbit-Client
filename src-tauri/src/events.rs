//! Имена Tauri-событий, которые бэкенд отправляет во фронтенд.
//!
//! Держим их в одном месте, чтобы Rust и JS не расходились в строках.

/// `{ user_code, verification_uri, expires_in, interval }`
pub const AUTH_DEVICE_CODE: &str = "auth://device-code";

/// `{ state: "pending" | "success" | "error", message }`
pub const AUTH_STATUS: &str = "auth://status";

/// `{ task_id, file_name, bytes_done, bytes_total }`
pub const DOWNLOAD_PROGRESS: &str = "download://progress";

/// `{ slug, state, message }`
pub const INSTANCE_OPTIMIZATION: &str = "instance://optimization";

/// `{ instance_id, line }`
pub const LOG_LINE: &str = "log://line";
