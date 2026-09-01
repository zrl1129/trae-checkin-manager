use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraeInstance {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub account_id: String,
    pub data_dir: String,
    pub debug_port: u16,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub last_launched_at: i64,
    #[serde(default)]
    pub last_closed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Default for TraeInstance {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            account_id: String::new(),
            data_dir: String::new(),
            debug_port: 9222,
            note: None,
            is_default: false,
            machine_id: None,
            last_launched_at: 0,
            last_closed_at: 0,
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceStore {
    pub instances: Vec<TraeInstance>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceBrief {
    pub id: String,
    pub name: String,
    pub data_dir: String,
    pub debug_port: u16,
    pub account_id: String,
    pub note: Option<String>,
    pub is_default: bool,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub disk_usage: u64,
    pub last_launched_at: i64,
    pub last_closed_at: i64,
    pub created_at: i64,
}
