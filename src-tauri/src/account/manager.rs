use std::sync::Mutex;

use anyhow::Result;

use crate::account::types::*;
use crate::storage;
use crate::trae::storage_reader::TraeLoginInfo;

pub struct AccountManager {
    store: Mutex<AccountStore>,
}

impl AccountManager {
    pub fn load() -> Result<Self> {
        let store = storage::read_json::<AccountStore>("accounts.json")?;
        Ok(Self {
            store: Mutex::new(store),
        })
    }

    pub fn get_all(&self) -> Vec<Account> {
        self.store.lock().unwrap().accounts.clone()
    }

    pub fn get(&self, id: &str) -> Option<Account> {
        self.store
            .lock()
            .unwrap()
            .accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
    }

    pub fn find_by_user_id(&self, user_id: &str) -> Option<Account> {
        self.store
            .lock()
            .unwrap()
            .accounts
            .iter()
            .find(|a| a.user_id == user_id)
            .cloned()
    }

    pub fn add(&self, name: String, email: String, note: Option<String>) -> Result<Account> {
        let mut store = self.store.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
            note,
            ..Default::default()
        };
        store.accounts.push(account.clone());
        storage::write_json("accounts.json", &*store)?;
        Ok(account)
    }

    pub fn add_from_local_login(&self, info: &TraeLoginInfo) -> Result<Account> {
        let mut store = self.store.lock().unwrap();
        if let Some(existing) = store.accounts.iter().find(|a| a.user_id == info.user_id) {
            return Ok(existing.clone());
        }
        let now = chrono::Utc::now().timestamp();
        let name = if info.username.is_empty() {
            format!("用户_{}", &info.user_id[..info.user_id.len().min(8)])
        } else {
            info.username.clone()
        };
        let account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email: info.email.clone(),
            note: Some("自动发现".to_string()),
            user_id: info.user_id.clone(),
            jwt_token: info.token.clone(),
            refresh_token: info.refresh_token.clone(),
            avatar_url: info.avatar_url.clone(),
            source: "local".to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        store.accounts.push(account.clone());
        storage::write_json("accounts.json", &*store)?;
        Ok(account)
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.accounts.retain(|a| a.id != id);
        storage::write_json("accounts.json", &*store)?;
        Ok(())
    }

    pub fn update(
        &self,
        id: &str,
        name: String,
        email: String,
        note: Option<String>,
    ) -> Result<Account> {
        let mut store = self.store.lock().unwrap();
        let account = store
            .accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| anyhow::anyhow!("账号不存在: {}", id))?;
        account.name = name;
        account.email = email;
        account.note = note;
        account.updated_at = chrono::Utc::now().timestamp();
        let updated = account.clone();
        storage::write_json("accounts.json", &*store)?;
        Ok(updated)
    }

    pub fn update_note(&self, id: &str, note: Option<String>) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(acc) = store.accounts.iter_mut().find(|a| a.id == id) {
            acc.note = note;
            acc.updated_at = chrono::Utc::now().timestamp();
        }
        storage::write_json("accounts.json", &*store)?;
        Ok(())
    }

    pub fn bind_instance(&self, account_id: &str, _instance_id: &str) -> Result<()> {
        Ok(())
    }

    pub fn export_accounts(&self) -> Result<String> {
        let store = self.store.lock().unwrap();
        let data = ExportData {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            accounts: store.accounts.clone(),
        };
        Ok(serde_json::to_string_pretty(&data)?)
    }

    pub fn import_accounts(&self, json_str: &str, overwrite: bool) -> Result<usize> {
        let parsed: ExportData = serde_json::from_str(json_str)
            .or_else(|_| -> anyhow::Result<ExportData> {
                let accs: Vec<Account> = serde_json::from_str(json_str)?;
                Ok(ExportData {
                    version: "1.0".to_string(),
                    exported_at: String::new(),
                    accounts: accs,
                })
            })?;
        let mut store = self.store.lock().unwrap();
        let count;
        if overwrite {
            let backup_path = storage::get_data_file("accounts.json.bak");
            let _ = std::fs::write(&backup_path, serde_json::to_string_pretty(&*store)?);
            store.accounts = parsed.accounts;
            count = store.accounts.len();
        } else {
            let existing: std::collections::HashSet<String> =
                store.accounts.iter().map(|a| a.user_id.clone()).collect();
            let mut added = 0;
            for acc in parsed.accounts {
                if !acc.user_id.is_empty() && !existing.contains(&acc.user_id) {
                    store.accounts.push(acc);
                    added += 1;
                } else if acc.user_id.is_empty() {
                    store.accounts.push(acc);
                    added += 1;
                }
            }
            count = added;
        }
        storage::write_json("accounts.json", &*store)?;
        Ok(count)
    }
}
