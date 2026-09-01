use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraeInstance {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub data_dir: String,
    pub debug_port: u16,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceStore {
    pub instances: Vec<TraeInstance>,
}
