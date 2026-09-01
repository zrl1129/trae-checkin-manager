use std::sync::Mutex;

use anyhow::Result;

use crate::account::manager::AccountManager;
use crate::instance::types::*;
use crate::storage;
use crate::trae::storage_reader;

pub struct InstanceManager {
    store: Mutex<InstanceStore>,
    disk_cache: Mutex<std::collections::HashMap<String, (i64, u64)>>,
}

impl InstanceManager {
    pub fn load() -> Result<Self> {
        let store = storage::read_json::<InstanceStore>("instances.json")?;
        Ok(Self {
            store: Mutex::new(store),
            disk_cache: Mutex::new(std::collections::HashMap::new()),
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
        self.create_with_dir(name, account_id, None, None)
    }

    pub fn create_with_dir(
        &self,
        name: String,
        account_id: String,
        data_dir: Option<String>,
        note: Option<String>,
    ) -> Result<TraeInstance> {
        let mut store = self.store.lock().unwrap();
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
        let data_dir = data_dir.unwrap_or_else(|| {
            format!("{}\\TRAE SOLO CN_{}", appdata, safe_name)
        });
        if store.instances.iter().any(|i| i.data_dir.eq_ignore_ascii_case(&data_dir)) {
            return Err(anyhow::anyhow!("数据目录已存在: {}", data_dir));
        }
        let port = find_available_port(&store.instances);
        let now = chrono::Utc::now().timestamp();
        let instance = TraeInstance {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            account_id,
            data_dir,
            debug_port: port,
            note,
            is_default: false,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        store.instances.push(instance.clone());
        storage::write_json("instances.json", &*store)?;
        Ok(instance)
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        self.remove_with_data(id, false)
    }

    pub fn remove_with_data(&self, id: &str, delete_data: bool) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter().find(|i| i.id == id) {
            if inst.is_default {
                return Err(anyhow::anyhow!("默认实例不可删除"));
            }
            if delete_data {
                let _ = std::fs::remove_dir_all(&inst.data_dir);
            }
        }
        store.instances.retain(|i| i.id != id);
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn rename(&self, id: &str, new_name: String) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.name = new_name;
            inst.updated_at = chrono::Utc::now().timestamp();
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn update_note(&self, id: &str, note: Option<String>) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.note = note;
            inst.updated_at = chrono::Utc::now().timestamp();
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn set_last_launched(&self, id: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.last_launched_at = chrono::Utc::now().timestamp();
            inst.updated_at = inst.last_launched_at;
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn set_last_closed(&self, id: &str) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.last_closed_at = chrono::Utc::now().timestamp();
            inst.updated_at = inst.last_closed_at;
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn bind_account(&self, id: &str, account_id: Option<String>) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(inst) = store.instances.iter_mut().find(|i| i.id == id) {
            inst.account_id = account_id.unwrap_or_default();
            inst.updated_at = chrono::Utc::now().timestamp();
        }
        storage::write_json("instances.json", &*store)?;
        Ok(())
    }

    pub fn get_briefs(&self) -> Vec<InstanceBrief> {
        let store = self.store.lock().unwrap();
        store
            .instances
            .iter()
            .map(|inst| {
                let pid = storage_reader::read_code_lock_pid(&inst.data_dir);
                let is_running = pid.map(|p| is_pid_alive(p)).unwrap_or(false);
                let disk_usage = self
                    .disk_cache
                    .lock()
                    .unwrap()
                    .get(&inst.data_dir)
                    .map(|(_, s)| *s)
                    .unwrap_or(0);
                InstanceBrief {
                    id: inst.id.clone(),
                    name: inst.name.clone(),
                    data_dir: inst.data_dir.clone(),
                    debug_port: inst.debug_port,
                    account_id: inst.account_id.clone(),
                    note: inst.note.clone(),
                    is_default: inst.is_default,
                    is_running,
                    pid,
                    disk_usage,
                    last_launched_at: inst.last_launched_at,
                    last_closed_at: inst.last_closed_at,
                    created_at: inst.created_at,
                }
            })
            .collect()
    }

    pub fn update_disk_cache(&self, data_dir: &str, size: u64) {
        let now = chrono::Utc::now().timestamp();
        self.disk_cache
            .lock()
            .unwrap()
            .insert(data_dir.to_string(), (now, size));
    }

    pub fn get_disk_usage_cached(&self, data_dir: &str) -> u64 {
        self.disk_cache
            .lock()
            .unwrap()
            .get(data_dir)
            .map(|(_, s)| *s)
            .unwrap_or(0)
    }

    pub fn auto_discover(account_mgr: &AccountManager) -> Result<Vec<TraeInstance>> {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let appdata_path = std::path::PathBuf::from(&appdata);
        let mut discovered = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&appdata_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "TRAE SOLO CN_SharedExtensions" {
                    continue;
                }
                let is_default = name == "TRAE SOLO CN";
                let is_multi = name.starts_with("TRAE SOLO CN_");
                if !is_default && !is_multi {
                    continue;
                }
                let data_dir = entry.path().to_string_lossy().to_string();
                let storage_path = std::path::Path::new(&data_dir)
                    .join("User")
                    .join("globalStorage")
                    .join("storage.json");
                if !storage_path.exists() {
                    continue;
                }
                discovered.push((data_dir, is_default));
            }
        }
        let mut store = storage::read_json::<InstanceStore>("instances.json")?;
        let mut new_instances = Vec::new();
        for (data_dir, is_default) in &discovered {
            if store.instances.iter().any(|i| i.data_dir.eq_ignore_ascii_case(data_dir)) {
                continue;
            }
            let login_info = storage_reader::read_login_from_dir(data_dir).ok().flatten();
            let (name, account_id) = if let Some(info) = &login_info {
                let acc = account_mgr.add_from_local_login(info).ok();
                let n = acc.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| {
                    format!("用户_{}", &info.user_id[..info.user_id.len().min(8)])
                });
                let aid = acc.map(|a| a.id).unwrap_or_default();
                (n, aid)
            } else {
                let dir_name = std::path::Path::new(data_dir)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                (dir_name, String::new())
            };
            let port = find_available_port(&store.instances);
            let now = chrono::Utc::now().timestamp();
            let inst = TraeInstance {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                account_id,
                data_dir: data_dir.clone(),
                debug_port: port,
                note: Some("自动发现".to_string()),
                is_default: *is_default,
                created_at: now,
                updated_at: now,
                ..Default::default()
            };
            store.instances.push(inst.clone());
            new_instances.push(inst);
        }
        if !new_instances.is_empty() {
            storage::write_json("instances.json", &store)?;
        }
        Ok(new_instances)
    }

    pub fn auto_bind_accounts(&self, account_mgr: &AccountManager) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        let mut changed = false;
        for inst in store.instances.iter_mut() {
            if inst.account_id.is_empty() {
                if let Ok(Some(info)) = storage_reader::read_login_from_dir(&inst.data_dir) {
                    if !info.user_id.is_empty() {
                        if let Some(acc) = account_mgr.find_by_user_id(&info.user_id) {
                            inst.account_id = acc.id;
                            inst.updated_at = chrono::Utc::now().timestamp();
                            changed = true;
                        }
                    }
                }
            }
        }
        if changed {
            storage::write_json("instances.json", &*store)?;
        }
        Ok(())
    }
}

fn find_available_port(instances: &[TraeInstance]) -> u16 {
    let used: std::collections::HashSet<u16> = instances.iter().map(|i| i.debug_port).collect();
    (9222..=9300)
        .find(|p| !used.contains(p))
        .unwrap_or(9222)
}

#[cfg(target_os = "windows")]
fn is_pid_alive(pid: u32) -> bool {
    let mut cmd = std::process::Command::new("tasklist");
    cmd.args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"]);
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let _ = cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("TRAE SOLO CN.exe") || stdout.contains("Trae.exe") || stdout.contains("Code.exe")
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
