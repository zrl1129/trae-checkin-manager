mod account;
mod checkin;
mod instance;
mod scheduler;
mod state;
mod storage;
mod trae;

use std::collections::HashMap;

use tauri::{Emitter, Manager};
use state::AppState;

use account::types::{Account, ExportData};
use checkin::types::{BatchSummary, CheckinEvent, CheckinRecord, CheckinStatus};
use instance::types::{InstanceBrief, TraeInstance};
use trae::storage_reader::{SafeCleanItem, TraeLoginInfo};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let auto_checkin = std::env::args().any(|a| a == "--auto-checkin");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(init_state())
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            add_account,
            remove_account,
            update_account,
            update_account_note,
            get_instances,
            get_instance_briefs,
            create_instance,
            remove_instance,
            rename_instance,
            update_instance_note,
            bind_account_to_instance,
            launch_instance,
            stop_instance,
            check_instance_running,
            perform_checkin,
            batch_checkin,
            get_checkin_records,
            find_trae_path,
            setup_scheduled_task,
            remove_scheduled_task,
            get_scheduled_task_status,
            auto_discover_instances,
            create_instance_shortcut,
            get_safe_clean_items,
            safe_clean_instance,
            read_local_account,
            export_accounts,
            import_accounts,
        ]);

    if auto_checkin {
        builder = builder.setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let accounts = state.accounts.get_all();
                log::info!("Auto checkin started, {} accounts", accounts.len());

                for acc in &accounts {
                    let instance = state.instances.get_by_account(&acc.id);
                    if instance.is_none() {
                        let _ = handle.emit("checkin-status", CheckinEvent {
                            account_id: acc.id.clone(),
                            account_name: acc.name.clone(),
                            status: CheckinStatus::Failed,
                            detail: "未关联实例".to_string(),
                            points: None,
                        });
                        continue;
                    }
                    {
                        let store = state.checkin_store.lock().unwrap();
                        if store.has_checked_today(&acc.id) {
                            let _ = handle.emit("checkin-status", CheckinEvent {
                                account_id: acc.id.clone(),
                                account_name: acc.name.clone(),
                                status: CheckinStatus::AlreadySigned,
                                detail: "今日已签到".to_string(),
                                points: None,
                            });
                            continue;
                        }
                    }
                    let _ = handle.emit("checkin-status", CheckinEvent {
                        account_id: acc.id.clone(),
                        account_name: acc.name.clone(),
                        status: CheckinStatus::InProgress,
                        detail: "正在签到...".to_string(),
                        points: None,
                    });
                    let record = do_checkin(&acc.id, &state).await;
                    let _ = handle.emit("checkin-status", CheckinEvent {
                        account_id: acc.id.clone(),
                        account_name: acc.name.clone(),
                        status: record.status.clone(),
                        detail: record.detail.clone(),
                        points: record.points,
                    });
                }
                log::info!("Auto checkin complete");
            });
            Ok(())
        });
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_state() -> AppState {
    let accounts = account::manager::AccountManager::load()
        .expect("Failed to load accounts");

    if let Err(e) = instance::manager::InstanceManager::auto_discover(&accounts) {
        log::warn!("Auto discover failed: {}", e);
    }

    let instances = instance::manager::InstanceManager::load()
        .expect("Failed to load instances");

    let checkin_store = storage::read_json::<checkin::types::CheckinStore>("checkin.json")
        .unwrap_or_default();

    if let Err(e) = instances.auto_bind_accounts(&accounts) {
        log::warn!("Auto bind failed: {}", e);
    }

    AppState {
        accounts,
        instances,
        checkin_store: std::sync::Mutex::new(checkin_store),
        pids: std::sync::Mutex::new(HashMap::new()),
    }
}

// === Account Commands ===

#[tauri::command]
fn get_accounts(state: tauri::State<'_, AppState>) -> Vec<Account> {
    state.accounts.get_all()
}

#[tauri::command]
fn add_account(name: String, email: String, note: Option<String>, state: tauri::State<'_, AppState>) -> Result<Account, String> {
    state.accounts.add(name, email, note).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_account(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.accounts.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_account(id: String, name: String, email: String, note: Option<String>, state: tauri::State<'_, AppState>) -> Result<Account, String> {
    state.accounts.update(&id, name, email, note).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_account_note(id: String, note: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.accounts.update_note(&id, note).map_err(|e| e.to_string())
}

// === Instance Commands ===

#[tauri::command]
fn get_instances(state: tauri::State<'_, AppState>) -> Vec<TraeInstance> {
    state.instances.get_all()
}

#[tauri::command]
fn get_instance_briefs(state: tauri::State<'_, AppState>) -> Vec<InstanceBrief> {
    state.instances.get_briefs()
}

#[tauri::command]
fn create_instance(name: String, account_id: String, state: tauri::State<'_, AppState>) -> Result<TraeInstance, String> {
    state.instances.create(name, account_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_instance(id: String, delete_data: Option<bool>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.instances.remove_with_data(&id, delete_data.unwrap_or(false)).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_instance(id: String, new_name: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.instances.rename(&id, new_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_instance_note(id: String, note: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.instances.update_note(&id, note).map_err(|e| e.to_string())
}

#[tauri::command]
fn bind_account_to_instance(id: String, account_id: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.instances.bind_account(&id, account_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_instance(id: String, state: tauri::State<'_, AppState>) -> Result<u32, String> {
    let instance = state.instances.get(&id).ok_or_else(|| format!("实例不存在: {}", id))?;

    if let Some(pid) = trae::storage_reader::read_code_lock_pid(&instance.data_dir) {
        if trae::process::is_process_running(pid) {
            let _ = trae::process::kill_trae_pid(pid);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    let exe_path = trae::path::find_trae_exe().ok_or_else(|| "未找到 TRAE 可执行文件".to_string())?;
    let pid = trae::process::launch_trae(&exe_path, &instance.data_dir, instance.debug_port).map_err(|e| e.to_string())?;
    state.pids.lock().unwrap().insert(id.clone(), pid);
    let _ = state.instances.set_last_launched(&id);
    trae::process::wait_for_debug_port(instance.debug_port, 30000).await.map_err(|e| e.to_string())?;
    Ok(pid)
}

#[tauri::command]
async fn stop_instance(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pid = state.pids.lock().unwrap().remove(&id);
    if let Some(pid) = pid {
        trae::process::kill_trae_pid(pid).map_err(|e| e.to_string())?;
    }
    let _ = state.instances.set_last_closed(&id);
    Ok(())
}

#[tauri::command]
async fn check_instance_running(id: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let instance = state.instances.get(&id).ok_or_else(|| "实例不存在".to_string())?;
    let pid = trae::storage_reader::read_code_lock_pid(&instance.data_dir);
    Ok(pid.map(|p| {
        let alive = is_pid_alive_tauri(p);
        if alive { true } else { false }
    }).unwrap_or(false))
}

fn is_pid_alive_tauri(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("tasklist");
        cmd.args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"]);
        use std::os::windows::process::CommandExt;
        let _ = cmd.creation_flags(0x08000000);
        cmd.output()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                !stdout.trim().is_empty() && !stdout.contains("信息")
            })
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

// === Checkin Commands ===

#[tauri::command]
async fn perform_checkin(account_id: String, app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<CheckinRecord, String> {
    let account = state.accounts.get(&account_id).ok_or_else(|| "账号不存在".to_string())?;
    let account_name = account.name.clone();
    let _ = app.emit("checkin-status", CheckinEvent {
        account_id: account_id.clone(),
        account_name,
        status: CheckinStatus::InProgress,
        detail: "正在签到...".to_string(),
        points: None,
    });
    let record = do_checkin(&account_id, &state).await;
    let _ = app.emit("checkin-status", CheckinEvent {
        account_id: account_id.clone(),
        account_name: account.name.clone(),
        status: record.status.clone(),
        detail: record.detail.clone(),
        points: record.points,
    });
    Ok(record)
}

#[tauri::command]
async fn batch_checkin(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<BatchSummary, String> {
    let accounts = state.accounts.get_all();
    let total = accounts.len();
    let mut success = 0;
    let mut already_signed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for acc in &accounts {
        let instance = state.instances.get_by_account(&acc.id);
        if instance.is_none() {
            skipped += 1;
            let _ = app.emit("checkin-status", CheckinEvent {
                account_id: acc.id.clone(),
                account_name: acc.name.clone(),
                status: CheckinStatus::Failed,
                detail: "未关联实例".to_string(),
                points: None,
            });
            continue;
        }
        {
            let store = state.checkin_store.lock().unwrap();
            if store.has_checked_today(&acc.id) {
                already_signed += 1;
                let _ = app.emit("checkin-status", CheckinEvent {
                    account_id: acc.id.clone(),
                    account_name: acc.name.clone(),
                    status: CheckinStatus::AlreadySigned,
                    detail: "今日已签到".to_string(),
                    points: None,
                });
                continue;
            }
        }
        let _ = app.emit("checkin-status", CheckinEvent {
            account_id: acc.id.clone(),
            account_name: acc.name.clone(),
            status: CheckinStatus::InProgress,
            detail: "正在签到...".to_string(),
            points: None,
        });
        let record = do_checkin(&acc.id, &state).await;
        let _ = app.emit("checkin-status", CheckinEvent {
            account_id: acc.id.clone(),
            account_name: acc.name.clone(),
            status: record.status.clone(),
            detail: record.detail.clone(),
            points: record.points,
        });
        match record.status {
            CheckinStatus::Success => success += 1,
            CheckinStatus::AlreadySigned => already_signed += 1,
            CheckinStatus::NotLoggedIn | CheckinStatus::Failed => failed += 1,
            _ => {}
        }
    }

    Ok(BatchSummary { total, success, already_signed, failed, skipped })
}

async fn do_checkin(account_id: &str, state: &tauri::State<'_, AppState>) -> CheckinRecord {
    let instance = match state.instances.get_by_account(account_id) {
        Some(inst) => inst,
        None => return CheckinRecord {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            instance_id: String::new(),
            status: CheckinStatus::Failed,
            detail: "未关联实例".to_string(),
            points: None, checkin_time: None,
            created_at: chrono::Utc::now().timestamp(),
        },
    };

    if !trae::process::is_debug_port_open(instance.debug_port).await {
        if let Some(pid) = trae::storage_reader::read_code_lock_pid(&instance.data_dir) {
            if trae::process::is_process_running(pid) {
                log::info!("TRAE already running (pid={}), killing to relaunch with debug port", pid);
                let _ = trae::process::kill_trae_pid(pid);
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        }

        let exe_path = match trae::path::find_trae_exe() {
            Some(p) => p,
            None => return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: "未找到 TRAE 可执行文件".to_string(),
                points: None, checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            },
        };
        match trae::process::launch_trae(&exe_path, &instance.data_dir, instance.debug_port) {
            Ok(pid) => { state.pids.lock().unwrap().insert(instance.id.clone(), pid); }
            Err(e) => return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: format!("启动 TRAE 失败: {}", e),
                points: None, checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            },
        }
        if let Err(e) = trae::process::wait_for_debug_port(instance.debug_port, 30000).await {
            return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: format!("等待调试端口超时: {}", e),
                points: None, checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            };
        }
    }

    let mut cdp = match checkin::cdp::CdpClient::connect(instance.debug_port).await {
        Ok(c) => c,
        Err(e) => return CheckinRecord {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            instance_id: instance.id.clone(),
            status: CheckinStatus::Failed,
            detail: format!("CDP 连接失败: {}", e),
            points: None, checkin_time: None,
            created_at: chrono::Utc::now().timestamp(),
        },
    };

    let result = match checkin::flow::perform_checkin(&mut cdp).await {
        Ok(r) => r,
        Err(e) => {
            cdp.close().await;
            return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: format!("签到流程错误: {}", e),
                points: None, checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            };
        }
    };
    cdp.close().await;

    let now = chrono::Utc::now().timestamp();
    let record = CheckinRecord {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        instance_id: instance.id.clone(),
        status: result.status,
        detail: result.detail,
        points: result.points,
        checkin_time: Some(now),
        created_at: now,
    };
    {
        let mut store = state.checkin_store.lock().unwrap();
        store.records.push(record.clone());
        let _ = storage::write_json("checkin.json", &*store);
    }
    record
}

#[tauri::command]
fn get_checkin_records(state: tauri::State<'_, AppState>) -> Vec<CheckinRecord> {
    state.checkin_store.lock().unwrap().records.clone()
}

// === Discovery / Shortcut / Clean ===

#[tauri::command]
fn auto_discover_instances(state: tauri::State<'_, AppState>) -> Result<Vec<TraeInstance>, String> {
    instance::manager::InstanceManager::auto_discover(&state.accounts).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_instance_shortcut(id: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let inst = state.instances.get(&id).ok_or_else(|| "实例不存在".to_string())?;
    let exe_path = trae::path::find_trae_exe().ok_or_else(|| "未找到 TRAE 可执行文件".to_string())?;
    let exe_str = exe_path.display().to_string();
    let desktop = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let lnk_path = format!("{}\\Desktop\\TRAE - {}.lnk", desktop, inst.name);
    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = '--user-data-dir=\"{}\" --remote-debugging-port={}; $s.IconLocation = \"{}\"; $s.WorkingDirectory = \"{}\"; $s.Save()",
        lnk_path, exe_str, inst.data_dir, inst.debug_port, exe_str,
        exe_path.parent().map(|p| p.display().to_string()).unwrap_or_default()
    );
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = cmd.creation_flags(0x08000000);
    }
    cmd.output().map_err(|e| e.to_string())?;
    Ok(lnk_path)
}

#[tauri::command]
fn get_safe_clean_items(id: String, state: tauri::State<'_, AppState>) -> Result<Vec<SafeCleanItem>, String> {
    let inst = state.instances.get(&id).ok_or_else(|| "实例不存在".to_string())?;
    Ok(trae::storage_reader::get_safe_clean_items(&inst.data_dir))
}

#[tauri::command]
fn safe_clean_instance(id: String, keys: Vec<String>, state: tauri::State<'_, AppState>) -> Result<u64, String> {
    let inst = state.instances.get(&id).ok_or_else(|| "实例不存在".to_string())?;
    trae::storage_reader::safe_clean(&inst.data_dir, &keys).map_err(|e| e.to_string())
}

// === Local Account Reading ===

#[tauri::command]
fn read_local_account(state: tauri::State<'_, AppState>) -> Result<Option<Account>, String> {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let data_dir = format!("{}\\TRAE SOLO CN", appdata);
    match trae::storage_reader::read_login_from_dir(&data_dir) {
        Ok(Some(info)) => {
            if let Some(acc) = state.accounts.find_by_user_id(&info.user_id) {
                return Ok(Some(acc));
            }
            match state.accounts.add_from_local_login(&info) {
                Ok(acc) => Ok(Some(acc)),
                Err(e) => Err(e.to_string()),
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// === Import/Export ===

#[tauri::command]
fn export_accounts(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.accounts.export_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
fn import_accounts(json_str: String, overwrite: Option<bool>, state: tauri::State<'_, AppState>) -> Result<usize, String> {
    state.accounts.import_accounts(&json_str, overwrite.unwrap_or(false)).map_err(|e| e.to_string())
}

// === Path / Scheduler ===

#[tauri::command]
fn find_trae_path() -> String {
    trae::path::find_trae_exe().map(|p| p.display().to_string()).unwrap_or_else(|| "未找到".to_string())
}

#[tauri::command]
fn setup_scheduled_task(hour: u32, minute: u32) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    scheduler::create_task(&exe_path.display().to_string(), hour, minute).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_scheduled_task() -> Result<(), String> {
    scheduler::remove_task().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scheduled_task_status() -> Result<bool, String> {
    scheduler::task_exists().map_err(|e| e.to_string())
}
