use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::AppResult;

/// Все пути приложения в одном месте.
///
/// Инстансы адресуются только по `Uuid`, поэтому имя инстанса не может
/// использоваться для выхода за пределы каталога (`../`).
#[derive(Debug, Clone)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.root.join("instances")
    }

    pub fn instance_dir(&self, id: Uuid) -> PathBuf {
        self.instances_dir().join(id.to_string())
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.root.join("accounts.json")
    }

    pub fn tokens_dir(&self) -> PathBuf {
        self.root.join("tokens")
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Создаёт базовую структуру каталогов, если её ещё нет.
    pub fn ensure_layout(&self) -> AppResult<()> {
        std::fs::create_dir_all(self.instances_dir())?;
        Ok(())
    }
}
