use crate::auth::accounts::AccountStore;
use crate::error::AppResult;
use crate::paths::AppPaths;

/// User-Agent для всех исходящих запросов к Modrinth и Mojang.
/// Требование Modrinth API — идентифицировать клиент.
pub const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/orbit-client)"
);

/// Общее состояние приложения, доступное из Tauri-команд.
pub struct AppState {
    paths: AppPaths,
    http: reqwest::Client,
    accounts: AccountStore,
}

impl AppState {
    pub fn new(paths: AppPaths) -> AppResult<Self> {
        let http = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
        let accounts = AccountStore::load(paths.accounts_file())?;

        Ok(Self {
            paths,
            http,
            accounts,
        })
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn accounts(&self) -> &AccountStore {
        &self.accounts
    }
}
