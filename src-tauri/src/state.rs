use std::collections::HashMap;
use std::sync::Mutex;

use crate::account::manager::AccountManager;
use crate::checkin::types::CheckinStore;
use crate::instance::manager::InstanceManager;

pub struct AppState {
    pub accounts: AccountManager,
    pub instances: InstanceManager,
    pub checkin_store: Mutex<CheckinStore>,
    pub pids: Mutex<HashMap<String, u32>>,
}
