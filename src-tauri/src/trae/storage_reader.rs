use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use super::crypto;

#[derive(Debug, Clone, Deserialize)]
pub struct TraeLoginInfo {
    pub user_id: String,
    pub token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub region: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthAccount {
    #[serde(default)]
    email: String,
    #[serde(default)]
    avatar_url: String,
    #[serde(default)]
    username: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthInfo {
    #[serde(default)]
    token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    account: Option<AuthAccount>,
    #[serde(default)]
    host: String,
    #[serde(default, rename = "userRegion")]
    user_region: Option<UserRegion>,
}

#[derive(Debug, Clone, Deserialize)]
struct UserRegion {
    #[serde(default)]
    region: String,
}

pub fn read_login_from_dir(data_dir: &str) -> Result<Option<TraeLoginInfo>> {
    let storage_path = Path::new(data_dir)
        .join("User")
        .join("globalStorage")
        .join("storage.json");
    if !storage_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&storage_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let auth_key = "iCubeAuthInfo://icube.cloudide";
    let auth_value = json.get(auth_key).and_then(|v| v.as_str());
    let Some(auth_str) = auth_value else {
        return Ok(None);
    };
    let auth_str = auth_str.trim();
    let parsed: serde_json::Value = if auth_str.starts_with('{') {
        serde_json::from_str(auth_str)?
    } else {
        let decrypted = crypto::decrypt_storage_value(auth_str)?;
        serde_json::from_str(&decrypted)?
    };
    let info: AuthInfo = serde_json::from_value(parsed)?;
    if info.token.is_empty() || info.user_id.is_empty() {
        return Ok(None);
    }
    let (email, avatar_url, username) = if let Some(acc) = &info.account {
        (acc.email.clone(), acc.avatar_url.clone(), acc.username.clone())
    } else {
        (String::new(), String::new(), String::new())
    };
    let region = info
        .user_region
        .map(|r| r.region)
        .unwrap_or_else(|| if info.host.contains("sg") { "SG".to_string() } else { "CN".to_string() });
    Ok(Some(TraeLoginInfo {
        user_id: info.user_id,
        token: info.token,
        refresh_token: info.refresh_token,
        email,
        username,
        avatar_url,
        region,
    }))
}

pub fn read_code_lock_pid(data_dir: &str) -> Option<u32> {
    let lock_path = Path::new(data_dir).join("code.lock");
    let content = std::fs::read_to_string(&lock_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32)
}

pub fn get_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += get_dir_size(&p);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SafeCleanItem {
    pub key: String,
    pub label: String,
    pub category: String,
    pub path: String,
    pub size: u64,
}

pub fn get_safe_clean_items(data_dir: &str) -> Vec<SafeCleanItem> {
    let base = Path::new(data_dir);
    let candidates = [
        ("logs", "运行日志", "logs", "logs"),
        ("cache_cache", "网络缓存", "cache", "Cache"),
        ("cache_code_cache", "代码缓存", "cache", "Code Cache"),
        ("cache_cached_data", "缓存数据", "cache", "CachedData"),
        ("cache_gpu", "GPU 缓存", "cache", "GPUCache"),
        ("crash_reports", "崩溃转储", "crash", "Crashpad/reports"),
    ];
    candidates
        .iter()
        .filter_map(|(key, label, category, rel_path)| {
            let full = base.join(rel_path);
            if full.exists() {
                let size = get_dir_size(&full);
                Some(SafeCleanItem {
                    key: key.to_string(),
                    label: label.to_string(),
                    category: category.to_string(),
                    path: full.display().to_string(),
                    size,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn safe_clean(data_dir: &str, keys: &[String]) -> Result<u64> {
    let items = get_safe_clean_items(data_dir);
    let mut freed: u64 = 0;
    for key in keys {
        if let Some(item) = items.iter().find(|i| &i.key == key) {
            let p = std::path::PathBuf::from(&item.path);
            if p.is_dir() {
                freed += item.size;
                if let Err(e) = std::fs::remove_dir_all(&p) {
                    log::warn!("Failed to clean {}: {}", item.path, e);
                }
            }
        }
    }
    Ok(freed)
}
