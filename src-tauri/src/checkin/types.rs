use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckinStatus {
    Pending,
    InProgress,
    Success,
    AlreadySigned,
    NotLoggedIn,
    Failed,
}

impl std::fmt::Display for CheckinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckinStatus::Pending => write!(f, "待签到"),
            CheckinStatus::InProgress => write!(f, "签到中"),
            CheckinStatus::Success => write!(f, "已签到"),
            CheckinStatus::AlreadySigned => write!(f, "今日已签"),
            CheckinStatus::NotLoggedIn => write!(f, "未登录"),
            CheckinStatus::Failed => write!(f, "失败"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRecord {
    pub id: String,
    pub account_id: String,
    pub instance_id: String,
    pub status: CheckinStatus,
    pub detail: String,
    pub points: Option<i64>,
    pub checkin_time: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckinStore {
    pub records: Vec<CheckinRecord>,
}
