use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub jwt_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub token_expired_at: Option<String>,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub plan_type: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub cookies: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            email: String::new(),
            note: None,
            user_id: String::new(),
            jwt_token: String::new(),
            refresh_token: String::new(),
            token_expired_at: None,
            avatar_url: String::new(),
            plan_type: String::new(),
            source: String::new(),
            cookies: String::new(),
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub accounts: Vec<Account>,
}
