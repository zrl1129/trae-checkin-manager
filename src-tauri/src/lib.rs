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

use account::types::Account;
use checkin::types::{BatchSummary, CheckinEvent, CheckinRecord, CheckinStatus};
use instance::types::TraeInstance;

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
            get_instances,
            create_instance,
            remove_instance,
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
        ]);

    if auto_checkin {
        builder = builder.setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let accounts = state.accounts.get_all();
                log::info!("自动签到启动，共 {} 个账号", accounts.len());

                for acc in &accounts {
                    let instance = state.instances.get_by_account(&acc.id);
                    if instance.is_none() {
                        let _ = handle.emit(
                            "checkin-status",
                            CheckinEvent {
                                account_id: acc.id.clone(),
                                account_name: acc.name.clone(),
                                status: CheckinStatus::Failed,
                                detail: "未关联实例".to_string(),
                                points: None,
                            },
                        );
                        continue;
                    }

                    {
                        let store = state.checkin_store.lock().unwrap();
                        if store.has_checked_today(&acc.id) {
                            let _ = handle.emit(
                                "checkin-status",
                                CheckinEvent {
                                    account_id: acc.id.clone(),
                                    account_name: acc.name.clone(),
                                    status: CheckinStatus::AlreadySigned,
                                    detail: "今日已签到".to_string(),
                                    points: None,
                                },
                            );
                            continue;
                        }
                    }

                    let _ = handle.emit(
                        "checkin-status",
                        CheckinEvent {
                            account_id: acc.id.clone(),
                            account_name: acc.name.clone(),
                            status: CheckinStatus::InProgress,
                            detail: "正在签到...".to_string(),
                            points: None,
                        },
                    );

                    let record = do_checkin(&acc.id, &state).await;

                    let _ = handle.emit(
                        "checkin-status",
                        CheckinEvent {
                            account_id: acc.id.clone(),
                            account_name: acc.name.clone(),
                            status: record.status.clone(),
                            detail: record.detail.clone(),
                            points: record.points,
                        },
                    );
                }

                log::info!("自动签到完成");
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
    let instances = instance::manager::InstanceManager::load()
        .expect("Failed to load instances");
    let checkin_store = storage::read_json::<checkin::types::CheckinStore>("checkin.json")
        .unwrap_or_default();

    AppState {
        accounts,
        instances,
        checkin_store: std::sync::Mutex::new(checkin_store),
        pids: std::sync::Mutex::new(HashMap::new()),
    }
}

#[tauri::command]
fn get_accounts(state: tauri::State<'_, AppState>) -> Vec<Account> {
    state.accounts.get_all()
}

#[tauri::command]
fn add_account(
    name: String,
    email: String,
    note: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Account, String> {
    state
        .accounts
        .add(name, email, note)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_account(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.accounts.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_account(
    id: String,
    name: String,
    email: String,
    note: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Account, String> {
    state
        .accounts
        .update(&id, name, email, note)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_instances(state: tauri::State<'_, AppState>) -> Vec<TraeInstance> {
    state.instances.get_all()
}

#[tauri::command]
fn create_instance(
    name: String,
    account_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<TraeInstance, String> {
    state
        .instances
        .create(name, account_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_instance(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.instances.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_instance(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<u32, String> {
    let instance = state
        .instances
        .get(&id)
        .ok_or_else(|| format!("实例不存在: {}", id))?;

    let exe_path = trae::path::find_trae_exe()
        .ok_or_else(|| "未找到 TRAE 可执行文件，请设置 TRAE_EXE_PATH 环境变量".to_string())?;

    let pid = trae::process::launch_trae(&exe_path, &instance.data_dir, instance.debug_port)
        .map_err(|e| e.to_string())?;

    state.pids.lock().unwrap().insert(id.clone(), pid);

    trae::process::wait_for_debug_port(instance.debug_port, 30000)
        .await
        .map_err(|e| e.to_string())?;

    Ok(pid)
}

#[tauri::command]
async fn stop_instance(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pid = state.pids.lock().unwrap().remove(&id);
    if let Some(pid) = pid {
        trae::process::kill_trae_pid(pid).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn check_instance_running(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let pid = state.pids.lock().unwrap().get(&id).copied();
    Ok(pid.map(|p| trae::process::is_process_running(p)).unwrap_or(false))
}

#[tauri::command]
async fn perform_checkin(
    account_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<CheckinRecord, String> {
    let account = state
        .accounts
        .get(&account_id)
        .ok_or_else(|| "账号不存在".to_string())?;
    let account_name = account.name.clone();

    let _ = app.emit(
        "checkin-status",
        CheckinEvent {
            account_id: account_id.clone(),
            account_name: account_name.clone(),
            status: CheckinStatus::InProgress,
            detail: "正在签到...".to_string(),
            points: None,
        },
    );

    let record = do_checkin(&account_id, &state).await;

    let _ = app.emit(
        "checkin-status",
        CheckinEvent {
            account_id: account_id.clone(),
            account_name,
            status: record.status.clone(),
            detail: record.detail.clone(),
            points: record.points,
        },
    );

    Ok(record)
}

#[tauri::command]
async fn batch_checkin(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<BatchSummary, String> {
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
            let _ = app.emit(
                "checkin-status",
                CheckinEvent {
                    account_id: acc.id.clone(),
                    account_name: acc.name.clone(),
                    status: CheckinStatus::Failed,
                    detail: "未关联实例".to_string(),
                    points: None,
                },
            );
            continue;
        }

        {
            let store = state.checkin_store.lock().unwrap();
            if store.has_checked_today(&acc.id) {
                already_signed += 1;
                let _ = app.emit(
                    "checkin-status",
                    CheckinEvent {
                        account_id: acc.id.clone(),
                        account_name: acc.name.clone(),
                        status: CheckinStatus::AlreadySigned,
                        detail: "今日已签到".to_string(),
                        points: None,
                    },
                );
                continue;
            }
        }

        let _ = app.emit(
            "checkin-status",
            CheckinEvent {
                account_id: acc.id.clone(),
                account_name: acc.name.clone(),
                status: CheckinStatus::InProgress,
                detail: "正在签到...".to_string(),
                points: None,
            },
        );

        let record = do_checkin(&acc.id, &state).await;

        let _ = app.emit(
            "checkin-status",
            CheckinEvent {
                account_id: acc.id.clone(),
                account_name: acc.name.clone(),
                status: record.status.clone(),
                detail: record.detail.clone(),
                points: record.points,
            },
        );

        match record.status {
            CheckinStatus::Success => success += 1,
            CheckinStatus::AlreadySigned => already_signed += 1,
            CheckinStatus::NotLoggedIn | CheckinStatus::Failed => failed += 1,
            _ => {}
        }
    }

    Ok(BatchSummary {
        total,
        success,
        already_signed,
        failed,
        skipped,
    })
}

async fn do_checkin(
    account_id: &str,
    state: &tauri::State<'_, AppState>,
) -> CheckinRecord {
    let instance = match state.instances.get_by_account(account_id) {
        Some(inst) => inst,
        None => {
            return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: String::new(),
                status: CheckinStatus::Failed,
                detail: "未关联实例".to_string(),
                points: None,
                checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            };
        }
    };

    if !trae::process::is_debug_port_open(instance.debug_port).await {
        let exe_path = match trae::path::find_trae_exe() {
            Some(p) => p,
            None => {
                return CheckinRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: account_id.to_string(),
                    instance_id: instance.id.clone(),
                    status: CheckinStatus::Failed,
                    detail: "未找到 TRAE 可执行文件".to_string(),
                    points: None,
                    checkin_time: None,
                    created_at: chrono::Utc::now().timestamp(),
                };
            }
        };

        match trae::process::launch_trae(&exe_path, &instance.data_dir, instance.debug_port) {
            Ok(pid) => {
                state
                    .pids
                    .lock()
                    .unwrap()
                    .insert(instance.id.clone(), pid);
            }
            Err(e) => {
                return CheckinRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: account_id.to_string(),
                    instance_id: instance.id.clone(),
                    status: CheckinStatus::Failed,
                    detail: format!("启动 TRAE 失败: {}", e),
                    points: None,
                    checkin_time: None,
                    created_at: chrono::Utc::now().timestamp(),
                };
            }
        }

        if let Err(e) = trae::process::wait_for_debug_port(instance.debug_port, 30000).await {
            return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: format!("等待调试端口超时: {}", e),
                points: None,
                checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            };
        }
    }

    let mut cdp = match checkin::cdp::CdpClient::connect(instance.debug_port).await {
        Ok(c) => c,
        Err(e) => {
            return CheckinRecord {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                instance_id: instance.id.clone(),
                status: CheckinStatus::Failed,
                detail: format!("CDP 连接失败: {}", e),
                points: None,
                checkin_time: None,
                created_at: chrono::Utc::now().timestamp(),
            };
        }
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
                points: None,
                checkin_time: None,
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
    state
        .checkin_store
        .lock()
        .unwrap()
        .records
        .clone()
}

#[tauri::command]
fn find_trae_path() -> String {
    trae::path::find_trae_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "未找到".to_string())
}

#[tauri::command]
fn setup_scheduled_task(
    hour: u32,
    minute: u32,
) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| e.to_string())?;
    scheduler::create_task(&exe_path.display().to_string(), hour, minute)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_scheduled_task() -> Result<(), String> {
    scheduler::remove_task().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scheduled_task_status() -> Result<bool, String> {
    scheduler::task_exists().map_err(|e| e.to_string())
}
