mod account;
mod checkin;
mod instance;
mod state;
mod storage;
mod trae;

use std::collections::HashMap;

use state::AppState;

use account::types::Account;
use checkin::types::CheckinRecord;
use instance::types::TraeInstance;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            get_checkin_records,
            find_trae_path,
        ])
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
    state: tauri::State<'_, AppState>,
) -> Result<CheckinRecord, String> {
    let instance = state
        .instances
        .get_by_account(&account_id)
        .ok_or_else(|| "该账号没有关联的实例，请先创建实例".to_string())?;

    if !trae::process::is_debug_port_open(instance.debug_port).await {
        let exe_path = trae::path::find_trae_exe()
            .ok_or_else(|| "未找到 TRAE 可执行文件".to_string())?;

        let pid = trae::process::launch_trae(&exe_path, &instance.data_dir, instance.debug_port)
            .map_err(|e| e.to_string())?;

        state
            .pids
            .lock()
            .unwrap()
            .insert(instance.id.clone(), pid);

        trae::process::wait_for_debug_port(instance.debug_port, 30000)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut cdp = checkin::cdp::CdpClient::connect(instance.debug_port)
        .await
        .map_err(|e| e.to_string())?;

    let result = checkin::flow::perform_checkin(&mut cdp)
        .await
        .map_err(|e| e.to_string())?;

    cdp.close().await;

    let now = chrono::Utc::now().timestamp();
    let record = CheckinRecord {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.clone(),
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
        storage::write_json("checkin.json", &*store).map_err(|e| e.to_string())?;
    }

    Ok(record)
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
