use std::sync::Mutex;

use anyhow::Result;

use crate::instance::types::*;
use crate::storage;

pub struct InstanceManager {
    store: Mutex<InstanceStore>,
}

impl InstanceManager {
    pub fn load() -> Result<Self> {
        let store = storage::read_json::<InstanceStore>("instances.json")?;
        Ok(Self {
            store: Mutex::new(store),
        })
    }

    pub fn get_all(&self) -> Vec<TraeInstance> {
        self.store.lock().unwrap().instances.clone()
    }

    pub fn get(&self, id: &str) -> Option<TraeInstance> {
        self.store
            .lock()
            .unwrap()
            .instances
            .iter()
            .find(|i| i.id == id)
            .cloned()
    }

    pub fn get_by_account(&self, account_id: &str) -> Option<TraeInstance> {
        self.store
            .lock()
            .unwrap()
            .instances
            .iter()
            .find(|i| i.account_id == account_id)
            .cloned()
    }

    pub fn create(&self, name: String, account_id: String) -> Result<TraeInstance> {
        let mut store = self.store.lock().unwrap();

        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
        let data_dir = format!("{}\\TRAE SOLO CN_{}", appdata, safe_name);

        let port = find_available_port(&store.instances);

        let now = chrono::Utc::now().timestamp();
        let instance = TraeInstance {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            account_id,
            data_dir,
            debug_port: port,
            created_at: now,
            updated_at: now,
        };
        store.instances.push(instance.clone());
        storage::write_json("instances.json", &*store)?;
        Ok(instance)
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.instances.retain(|i| i.id != id);
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn update_port(&self, id: &str, port: u16) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.debug_port = port;
            inst.updated_at = chrono::Utc::now().timestamp();
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }
}

fn find_available_port(instances: &[TraeInstance]) -> u16 {
    let used: std::collections::HashSet<u16> = instances.iter().map(|i| i.debug_port).collect();
    (9222..=9300)
        .find(|p| !used.contains(p))
        .unwrap_or(9222)
}
