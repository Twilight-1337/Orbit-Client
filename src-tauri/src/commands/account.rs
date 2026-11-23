use tauri::State;
use uuid::Uuid;

use crate::auth::accounts::AccountSnapshot;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub fn account_state(state: State<'_, AppState>) -> AppResult<AccountSnapshot> {
    state.accounts().snapshot()
}

/// Создание оффлайн-профиля. Шлюз проверяется здесь, в Rust, — отключённая
/// кнопка в UI защитой не является.
#[tauri::command]
pub fn create_offline_account(
    state: State<'_, AppState>,
    username: String,
) -> AppResult<AccountSnapshot> {
    state.accounts().create_offline(&username)?;
    state.accounts().snapshot()
}

#[tauri::command]
pub fn remove_account(state: State<'_, AppState>, account_id: Uuid) -> AppResult<AccountSnapshot> {
    state.accounts().remove(account_id)?;
    state.accounts().snapshot()
}

#[tauri::command]
pub fn set_active_account(
    state: State<'_, AppState>,
    account_id: Option<Uuid>,
) -> AppResult<AccountSnapshot> {
    state.accounts().set_active(account_id)?;
    state.accounts().snapshot()
}

/// Добавляет фиктивный лицензионный аккаунт, чтобы проверить шлюз без OAuth.
/// В release-сборке команда всегда возвращает ошибку.
#[tauri::command]
pub fn debug_seed_licensed_account(
    state: State<'_, AppState>,
    username: Option<String>,
) -> AppResult<AccountSnapshot> {
    if !cfg!(debug_assertions) {
        return Err(AppError::Internal(
            "debug commands are disabled in release builds".into(),
        ));
    }

    state
        .accounts()
        .seed_licensed_account(username.as_deref().unwrap_or("DevAccount"))?;
    state.accounts().snapshot()
}
