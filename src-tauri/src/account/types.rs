use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub note: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub accounts: Vec<Account>,
}
