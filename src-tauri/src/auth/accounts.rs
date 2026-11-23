//! Аккаунты и «лицензионный шлюз».
//!
//! Правило продукта: оффлайн-профили разрешены только пока в лаунчере есть
//! хотя бы один подтверждённый лицензионный Microsoft-аккаунт. Проверка живёт
//! здесь, на бэкенде, — UI лишь отображает её результат.

use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub const SCHEMA_VERSION: u32 = 1;
pub const MIN_USERNAME_LEN: usize = 3;
pub const MAX_USERNAME_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseInfo {
    pub owns_java_edition: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub verified_at: OffsetDateTime,
}

/// Сохраняемая модель аккаунта. Секретов здесь нет и быть не должно —
/// refresh-токены живут в `auth::token_store`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: Uuid,
    pub kind: AccountKind,
    pub username: String,
    pub mc_uuid: Uuid,
    pub skin_url: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub license: Option<LicenseInfo>,
}

impl Account {
    pub fn license_verified(&self) -> bool {
        self.license
            .as_ref()
            .is_some_and(|info| info.owns_java_edition)
    }
}

/// Представление аккаунта для фронтенда: содержит вычисленные поля.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: Uuid,
    pub kind: AccountKind,
    pub username: String,
    pub mc_uuid: Uuid,
    pub skin_url: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub license_verified: bool,
    /// `true`, если аккаунт существует, но запускать с ним нельзя.
    pub blocked: bool,
}

impl AccountView {
    fn new(account: &Account, has_license: bool) -> Self {
        let license_verified = account.license_verified();
        let blocked = match account.kind {
            AccountKind::Microsoft => !license_verified,
            AccountKind::Offline => !has_license,
        };

        Self {
            id: account.id,
            kind: account.kind,
            username: account.username.clone(),
            mc_uuid: account.mc_uuid,
            skin_url: account.skin_url.clone(),
            created_at: account.created_at,
            license_verified,
            blocked,
        }
    }
}

/// Снимок состояния аккаунтов — единственное, что фронтенд должен знать
/// о правилах доступа.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSnapshot {
    pub accounts: Vec<AccountView>,
    pub active_account_id: Option<Uuid>,
    pub has_license: bool,
    pub can_create_offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountsFile {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    accounts: Vec<Account>,
    #[serde(default)]
    active_account_id: Option<Uuid>,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

impl Default for AccountsFile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            accounts: Vec::new(),
            active_account_id: None,
        }
    }
}

/// Java-совместимый offline UUID.
///
/// Vanilla использует `UUID.nameUUIDFromBytes("OfflinePlayer:<name>".getBytes(UTF_8))`,
/// то есть MD5 без namespace. `Uuid::new_v3` с `NAMESPACE_OID` дал бы другой
/// результат, поэтому биты версии и варианта выставляем вручную.
pub fn offline_uuid(username: &str) -> Uuid {
    use md5::{Digest, Md5};

    let digest = Md5::digest(format!("OfflinePlayer:{username}").as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30; // version 3
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC 4122 variant
    Uuid::from_bytes(bytes)
}

/// Проверяет ник по правилам Minecraft: 3–16 символов, `[A-Za-z0-9_]`.
pub fn validate_username(raw: &str) -> AppResult<String> {
    let username = raw.trim();

    if !username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(AppError::InvalidData(
            "username may only contain letters, digits and underscores".into(),
        ));
    }

    let len = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&len) {
        return Err(AppError::InvalidData(format!(
            "username must be between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters"
        )));
    }

    Ok(username.to_string())
}

pub struct AccountStore {
    path: PathBuf,
    data: RwLock<AccountsFile>,
}

impl AccountStore {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let data = if path.exists() {
            let raw = std::fs::read(&path)?;
            serde_json::from_slice(&raw).map_err(|err| {
                AppError::InvalidData(format!("accounts.json is corrupted: {err}"))
            })?
        } else {
            AccountsFile::default()
        };

        Ok(Self {
            path,
            data: RwLock::new(data),
        })
    }

    /// `false` при отравленном локе — осознанный fail-closed: без уверенности
    /// в состоянии лицензии оффлайн-профили запрещены.
    pub fn has_license(&self) -> bool {
        match self.read() {
            Ok(data) => Self::has_license_in(&data),
            Err(_) => false,
        }
    }

    pub fn ensure_offline_allowed(&self) -> AppResult<()> {
        if self.has_license() {
            Ok(())
        } else {
            Err(AppError::LicenseRequired)
        }
    }

    pub fn snapshot(&self) -> AppResult<AccountSnapshot> {
        let data = self.read()?;
        let has_license = Self::has_license_in(&data);

        Ok(AccountSnapshot {
            accounts: data
                .accounts
                .iter()
                .map(|account| AccountView::new(account, has_license))
                .collect(),
            active_account_id: data.active_account_id,
            has_license,
            can_create_offline: has_license,
        })
    }

    pub fn create_offline(&self, raw_username: &str) -> AppResult<Account> {
        let username = validate_username(raw_username)?;

        let mut guard = self.write()?;
        let mut next = guard.clone();

        if !Self::has_license_in(&next) {
            return Err(AppError::LicenseRequired);
        }

        if next.accounts.iter().any(|account| {
            account.kind == AccountKind::Offline && account.username.eq_ignore_ascii_case(&username)
        }) {
            return Err(AppError::InvalidData(format!(
                "offline profile '{username}' already exists"
            )));
        }

        let account = Account {
            id: Uuid::new_v4(),
            kind: AccountKind::Offline,
            mc_uuid: offline_uuid(&username),
            username,
            skin_url: None,
            created_at: OffsetDateTime::now_utc(),
            license: None,
        };

        next.accounts.push(account.clone());
        if next.active_account_id.is_none() {
            next.active_account_id = Some(account.id);
        }

        self.commit(&mut guard, next)?;
        Ok(account)
    }

    /// Добавляет или обновляет Microsoft-аккаунт. Используется входом (шаг 3)
    /// и dev-командой для проверки шлюза без реального OAuth.
    pub fn upsert_microsoft(
        &self,
        mc_uuid: Uuid,
        raw_username: &str,
        skin_url: Option<String>,
        licensed: bool,
    ) -> AppResult<Account> {
        let username = validate_username(raw_username)?;

        let mut guard = self.write()?;
        let mut next = guard.clone();

        let license = licensed.then(|| LicenseInfo {
            owns_java_edition: true,
            verified_at: OffsetDateTime::now_utc(),
        });

        let account =
            match next.accounts.iter_mut().find(|account| {
                account.kind == AccountKind::Microsoft && account.mc_uuid == mc_uuid
            }) {
                Some(existing) => {
                    existing.username = username;
                    existing.skin_url = skin_url;
                    existing.license = license;
                    existing.clone()
                }
                None => {
                    let account = Account {
                        id: Uuid::new_v4(),
                        kind: AccountKind::Microsoft,
                        username,
                        mc_uuid,
                        skin_url,
                        created_at: OffsetDateTime::now_utc(),
                        license,
                    };
                    next.accounts.push(account.clone());
                    account
                }
            };

        if next.active_account_id.is_none() {
            next.active_account_id = Some(account.id);
        }

        self.commit(&mut guard, next)?;
        Ok(account)
    }

    /// Создаёт фиктивный лицензионный аккаунт, если лицензии ещё нет.
    /// Только для тестов и debug-сборок.
    pub fn seed_licensed_account(&self, raw_username: &str) -> AppResult<Account> {
        let existing = {
            let data = self.read()?;
            data.accounts
                .iter()
                .find(|account| {
                    account.kind == AccountKind::Microsoft && account.license_verified()
                })
                .cloned()
        };

        if let Some(account) = existing {
            return Ok(account);
        }

        self.upsert_microsoft(Uuid::new_v4(), raw_username, None, true)
    }

    pub fn remove(&self, account_id: Uuid) -> AppResult<()> {
        let mut guard = self.write()?;
        let mut next = guard.clone();

        let before = next.accounts.len();
        next.accounts.retain(|account| account.id != account_id);
        if next.accounts.len() == before {
            return Err(AppError::InvalidData(format!(
                "account {account_id} not found"
            )));
        }

        if next.active_account_id == Some(account_id) {
            next.active_account_id = next.accounts.first().map(|account| account.id);
        }

        self.commit(&mut guard, next)
    }

    pub fn set_active(&self, account_id: Option<Uuid>) -> AppResult<()> {
        let mut guard = self.write()?;
        let mut next = guard.clone();

        if let Some(id) = account_id {
            if !next.accounts.iter().any(|account| account.id == id) {
                return Err(AppError::InvalidData(format!("account {id} not found")));
            }
        }

        next.active_account_id = account_id;
        self.commit(&mut guard, next)
    }

    pub fn active_account(&self) -> AppResult<Option<Account>> {
        let data = self.read()?;
        Ok(data.active_account_id.and_then(|id| {
            data.accounts
                .iter()
                .find(|account| account.id == id)
                .cloned()
        }))
    }

    fn has_license_in(data: &AccountsFile) -> bool {
        data.accounts
            .iter()
            .any(|account| account.kind == AccountKind::Microsoft && account.license_verified())
    }

    /// Сначала пишем на диск, и только потом обновляем память — чтобы при
    /// ошибке записи состояние не разъехалось с файлом.
    fn commit(
        &self,
        guard: &mut RwLockWriteGuard<'_, AccountsFile>,
        next: AccountsFile,
    ) -> AppResult<()> {
        self.persist(&next)?;
        **guard = next;
        Ok(())
    }

    fn persist(&self, data: &AccountsFile) -> AppResult<()> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| AppError::Internal("accounts.json has no parent directory".into()))?;
        std::fs::create_dir_all(parent)?;

        let json = serde_json::to_vec_pretty(data)
            .map_err(|err| AppError::Internal(format!("failed to serialise accounts: {err}")))?;

        let mut tmp = self.path.clone().into_os_string();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);

        std::fs::write(&tmp, json)?;

        #[cfg(windows)]
        if self.path.exists() {
            // Windows не перезаписывает существующий файл через rename.
            std::fs::remove_file(&self.path)?;
        }

        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    fn read(&self) -> AppResult<RwLockReadGuard<'_, AccountsFile>> {
        self.data
            .read()
            .map_err(|_| AppError::Internal("account store lock poisoned".into()))
    }

    fn write(&self) -> AppResult<RwLockWriteGuard<'_, AccountsFile>> {
        self.data
            .write()
            .map_err(|_| AppError::Internal("account store lock poisoned".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store() -> (TempDir, AccountStore) {
        let dir = TempDir::new().expect("temp dir");
        let store = AccountStore::load(dir.path().join("accounts.json")).expect("load store");
        (dir, store)
    }

    #[test]
    fn offline_uuid_matches_java_implementation() {
        // Эталоны получены независимой реализацией (MD5 + биты версии 3).
        assert_eq!(
            offline_uuid("Notch").to_string(),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
        assert_eq!(
            offline_uuid("jeb_").to_string(),
            "a762f560-4fce-3236-812a-b80efff0b62b"
        );
        assert_eq!(
            offline_uuid("Player").to_string(),
            "a01e3843-e521-3998-958a-f459800e4d11"
        );
    }

    #[test]
    fn offline_profiles_are_blocked_without_license() {
        let (_dir, store) = store();

        assert!(!store.has_license());
        assert!(!store.snapshot().unwrap().can_create_offline);

        let err = store.create_offline("Steve").unwrap_err();
        assert!(matches!(err, AppError::LicenseRequired));
        assert_eq!(err.code(), "license_required");
    }

    #[test]
    fn license_unlocks_offline_profiles() {
        let (_dir, store) = store();
        store.seed_licensed_account("Licensed").unwrap();

        assert!(store.has_license());
        assert!(store.snapshot().unwrap().can_create_offline);

        let account = store.create_offline("Steve").unwrap();
        assert_eq!(account.kind, AccountKind::Offline);
        assert_eq!(account.username, "Steve");
        assert_eq!(account.mc_uuid, offline_uuid("Steve"));
    }

    #[test]
    fn removing_last_license_blocks_existing_offline_profiles() {
        let (_dir, store) = store();
        let licensed = store.seed_licensed_account("Licensed").unwrap();
        let offline = store.create_offline("Steve").unwrap();

        store.remove(licensed.id).unwrap();

        assert!(!store.has_license());
        assert!(store.ensure_offline_allowed().is_err());

        let snapshot = store.snapshot().unwrap();
        assert!(!snapshot.can_create_offline);
        let view = snapshot
            .accounts
            .iter()
            .find(|account| account.id == offline.id)
            .expect("offline account kept");
        assert!(view.blocked);

        // Профиль не удалён — он вернётся, когда лицензия появится снова.
        store.seed_licensed_account("Licensed").unwrap();
        let snapshot = store.snapshot().unwrap();
        assert!(
            !snapshot
                .accounts
                .iter()
                .find(|a| a.id == offline.id)
                .unwrap()
                .blocked
        );
    }

    #[test]
    fn unlicensed_microsoft_account_does_not_unlock_offline() {
        let (_dir, store) = store();
        store
            .upsert_microsoft(Uuid::new_v4(), "NoLicense", None, false)
            .unwrap();

        assert!(!store.has_license());
        assert!(matches!(
            store.create_offline("Steve").unwrap_err(),
            AppError::LicenseRequired
        ));

        let snapshot = store.snapshot().unwrap();
        assert!(snapshot.accounts.iter().all(|account| account.blocked));
    }

    #[test]
    fn usernames_are_validated() {
        let (_dir, store) = store();
        store.seed_licensed_account("Licensed").unwrap();

        assert!(matches!(
            store.create_offline("ab").unwrap_err(),
            AppError::InvalidData(_)
        ));
        assert!(matches!(
            store.create_offline("bad name").unwrap_err(),
            AppError::InvalidData(_)
        ));
        assert!(matches!(
            store.create_offline("abcdefghijklmnopq").unwrap_err(),
            AppError::InvalidData(_)
        ));

        // Пробелы по краям обрезаются, ник сохраняется как есть.
        assert_eq!(store.create_offline("  Alex  ").unwrap().username, "Alex");
    }

    #[test]
    fn duplicate_offline_username_is_rejected() {
        let (_dir, store) = store();
        store.seed_licensed_account("Licensed").unwrap();
        store.create_offline("Steve").unwrap();

        assert!(matches!(
            store.create_offline("steve").unwrap_err(),
            AppError::InvalidData(_)
        ));
    }

    #[test]
    fn state_survives_reload() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("accounts.json");

        let offline_id = {
            let store = AccountStore::load(path.clone()).unwrap();
            store.seed_licensed_account("Licensed").unwrap();
            store.create_offline("Steve").unwrap().id
        };

        let reloaded = AccountStore::load(path).unwrap();
        assert!(reloaded.has_license());
        let snapshot = reloaded.snapshot().unwrap();
        assert!(snapshot
            .accounts
            .iter()
            .any(|account| account.id == offline_id));
    }

    #[test]
    fn active_account_falls_back_after_removal() {
        let (_dir, store) = store();
        let licensed = store.seed_licensed_account("Licensed").unwrap();
        let offline = store.create_offline("Steve").unwrap();

        store.set_active(Some(offline.id)).unwrap();
        store.remove(offline.id).unwrap();

        let snapshot = store.snapshot().unwrap();
        assert_eq!(snapshot.active_account_id, Some(licensed.id));
        assert_eq!(store.active_account().unwrap().unwrap().id, licensed.id);
    }
}
