use std::sync::Mutex;

use anyhow::Result;

use crate::account::types::*;
use crate::storage;

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

    pub fn add(&self, name: String, email: String, note: Option<String>) -> Result<Account> {
        let mut store = self.store.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
            note,
            created_at: now,
            updated_at: now,
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
}
